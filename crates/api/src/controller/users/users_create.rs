use std::sync::Arc;

use chaty_proto::{
  users_create_response::Response::{Data, Error},
  UsersCreateRequest, UsersCreateResponse, UsersCreateResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName::UsersCreate, EventParameterKey, EventStatus::Fail},
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
  tr,
};
use serde_json::json;
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::{audit::process_audit, ApiController},
  models::users::users_create::{
    users_create_auditable, users_create_pre_save, users_create_validate,
  },
};

pub async fn users_create(
  ctr: &ApiController,
  request: Request<UsersCreateRequest>,
) -> Result<Response<UsersCreateResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();
  let path = "api.users.users_create";
  let req = request.into_inner();

  ctr.metrics.record_users_create_success();

  let mut audit = AuditRecord::new(ctx.clone(), UsersCreate, Fail);

  let users_clone = req.clone();
  let audit_future = spawn(async move { users_create_auditable(&users_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    let data = get_audit().await;
    audit_clone.set_event_parameter(EventParameterKey::UsersCreate, data);
    process_audit(&audit_clone);
    Response::new(UsersCreateResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = users_create_validate(ctx.clone(), path, &req) {
    ctr.metrics.record_users_create_failure();
    return Ok(return_err(err).await);
  }

  let user = match users_create_pre_save(ctx.clone(), path, &req).await {
    Ok(user) => user,
    Err(err) => {
      ctr.metrics.record_users_create_failure();
      return Ok(return_err(err).await);
    }
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("users_create");

  let db_res = ctr.sql_db.users_create(ctx.clone(), &user).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("users_create", db_duration);

  if let Err(err) = db_res {
    ctr.metrics.record_db_error("users_create", &err.msg);
    ctr.metrics.record_users_create_failure();
    let err_to_return = match err.err_type {
      ErrorType::ResourceExists => {
        // Check if it's email or username that exists
        let msg = if err.msg.contains("email") {
          "users.email.already_exists"
        } else {
          "users.username.already_exists"
        };
        AppError::new(ctx.clone(), path, msg, None, "", Code::AlreadyExists.into(), None)
      }
      _ => ie(Box::new(err)),
    };
    return Ok(return_err(err_to_return).await);
  }

  // Publish email confirmation message to broker
  let broker_start = std::time::Instant::now();
  let message = json!({ "user_id": user.id, "email": user.email });

  // TODO: Implement actual broker message publishing
  // For now, just record the intent
  ctr.metrics.record_broker_message_sent();
  let _broker_duration = broker_start.elapsed().as_secs_f64();

  let data = get_audit().await;
  audit.set_event_parameter(EventParameterKey::UsersCreate, data);
  audit.success();
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("users.users_create", request_duration);

  let message = tr::<()>(lang, "users.create.success", None)
    .unwrap_or_else(|_| "An account created successfully".to_string());

  Ok(Response::new(UsersCreateResponse {
    response: Some(Data(UsersCreateResponseData { message })),
  }))
}
