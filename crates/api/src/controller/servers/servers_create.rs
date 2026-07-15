use std::{sync::Arc, time::Instant};

use chaty_proto::{
  servers_create_response::Response::{Data, Error},
  ServerMember, ServersCreateRequest, ServersCreateResponse, ServersCreateResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName, EventParameterKey, EventStatus},
  context::Context,
  errors::{not_found_error, AppError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
};
use chaty_ws::v1::EventV1;
use serde_json::json;
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::{audit::process_audit, ApiController},
  models::servers::servers_create::{
    servers_create_auditable, servers_create_presave, servers_create_validate,
  },
};

pub async fn servers_create(
  ctr: &ApiController,
  request: Request<ServersCreateRequest>,
) -> Result<Response<ServersCreateResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.servers.servers_create";
  let req = request.into_inner();

  let mut audit = AuditRecord::new(ctx.clone(), EventName::ServersCreate, EventStatus::Fail);

  let req_clone = req.clone();
  let audit_future = spawn(async move { servers_create_auditable(&req_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    ctr.metrics.record_servers_create_failure();
    let data = get_audit().await;
    audit_clone.set_event_parameter(EventParameterKey::Data, data);
    process_audit(&audit_clone);
    Response::new(ServersCreateResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = servers_create_validate(ctx.clone(), path, &req) {
    return Ok(return_err(err).await);
  }

  let cache_start = Instant::now();
  let user = ctr.cache.users_get_or_insert_by_id(ctx.clone(), ctx.session().user_id()).await;
  let cache_duration = cache_start.elapsed().as_secs_f64();
  ctr.metrics.observe_cache_operation_duration("users_get_or_insert_by_id", cache_duration);
  if user.is_err() {
    let err = user.unwrap_err();
    if err.err_type == ErrorType::NoRows {
      return Ok(return_err(not_found_error(ctx.clone(), path, Some(Box::new(err)))).await);
    } else {
      return Ok(return_err(ie(Box::new(err))).await);
    }
  }
  let user = user.unwrap();

  let server_counts = ctr.nosql_db.server_members_count_for_user(&user.id).await;
  if server_counts.is_err() {
    return Ok(return_err(ie(Box::new(server_counts.unwrap_err()))).await);
  }
  let server_counts = server_counts.unwrap();

  let user_limits = ctr.nosql_db.users_get_limits(&user.id).await;
  if server_counts >= user_limits.servers as i64 {
    let params = ("servers.create.too_many_servers", Code::InvalidArgument.into());
    return Ok(
      return_err(AppError::new(ctx.clone(), path, params.0, None, "", params.1, None)).await,
    );
  }

  let (mut server, channels) = servers_create_presave(req, &user, true);
  if !channels.is_empty() {
    let db_res = ctr.nosql_db.channels_insert(&channels[0], ctx.session.user_id()).await;
    if db_res.is_err() {
      return Ok(return_err(ie(Box::new(db_res.unwrap_err()))).await);
    }
  }

  server.channels = channels.iter().map(|chan| chan.id.to_string()).collect();
  let mut db_result = ctr.nosql_db.servers_insert(ctx.clone(), &server).await;
  if db_result.is_err() {
    return Ok(return_err(ie(Box::new(db_result.unwrap_err()))).await);
  }

  let member =
    ServerMember { server_id: server.id.clone(), user_id: user.id, ..Default::default() };
  db_result = ctr.nosql_db.server_members_insert(ctx.clone(), &member).await;
  if db_result.is_err() {
    return Ok(return_err(ie(Box::new(db_result.unwrap_err()))).await);
  }

  let mut conn = match ctr.cache.get_conn(path).await {
    Ok(conn) => conn,
    Err(err) => return Ok(return_err(ie(Box::new(err))).await),
  };

  let server_id = server.id.clone();
  EventV1::ServerMemberJoin { id: server.id.clone(), member }.p(&mut conn, server_id.clone()).await;
  EventV1::ServerCreate {
    id: server_id.clone(),
    server: server,
    channels,
    emojis: vec![],
    voice_states: vec![],
  }
  .private(&mut conn, server_id)
  .await;

  audit.set_event_parameter(EventParameterKey::Data, get_audit().await);
  audit.success();
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("servers.servers_create", request_duration);
  ctr.metrics.record_servers_create_success();

  Ok(Response::new(ServersCreateResponse { response: Some(Data(ServersCreateResponseData {})) }))
}
