use std::sync::Arc;

use chaty_proto::{
  groups_create_response::Response::{Data, Error},
  GroupsCreateRequest, GroupsCreateResponse, GroupsCreateResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName::GroupsCreate, EventParameterKey, EventStatus::Fail},
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ERROR_ID_INTERNAL},
  tr,
};
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::{audit::process_audit, ApiController},
  models::groups::groups_create::{
    groups_create_auditable, groups_create_pre_save, groups_create_validate,
  },
};

pub async fn groups_create(
  ctr: &ApiController,
  request: Request<GroupsCreateRequest>,
) -> Result<Response<GroupsCreateResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();
  let path = "api.groups.groups_create";
  let req = request.into_inner();

  ctr.metrics.record_groups_create_success();

  let mut audit = AuditRecord::new(ctx.clone(), GroupsCreate, Fail);

  let req_clone = req.clone();
  let audit_future = spawn(async move { groups_create_auditable(&req_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| serde_json::json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    audit_clone.set_event_parameter(EventParameterKey::Data, get_audit().await);
    process_audit(&audit_clone);
    Response::new(GroupsCreateResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = groups_create_validate(ctx.clone(), path, &req) {
    ctr.metrics.record_groups_create_failure();
    return Ok(return_err(err).await);
  }

  let user_id = ctx.session.user_id();
  let channel = match groups_create_pre_save(ctx.clone(), path, &user_id, &req).await {
    Ok(channel) => channel,
    Err(err) => {
      ctr.metrics.record_groups_create_failure();
      return Ok(return_err(err).await);
    }
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("channels_groups_create");

  let db_res = ctr.nosql_db.channels_groups_create(ctx.clone(), &channel).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("channels_groups_create", db_duration);

  if let Err(err) = db_res {
    ctr.metrics.record_db_error("channels_groups_create", &err.msg);
    ctr.metrics.record_groups_create_failure();
    let err_to_return = ie(Box::new(err));
    return Ok(return_err(err_to_return).await);
  }

  audit.set_event_parameter(EventParameterKey::Data, get_audit().await);
  audit.success();
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("groups.groups_create", request_duration);

  let message = tr::<()>(lang, "groups.create.success", None)
    .unwrap_or_else(|_| "Group created successfully".to_string());

  Ok(Response::new(GroupsCreateResponse {
    response: Some(Data(GroupsCreateResponseData { message })),
  }))
}
