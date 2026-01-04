use std::io::{Error as StdErr, ErrorKind};
use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chaty_database::TokenType;
use chaty_proto::{
  users_reset_password_response::Response::{Data, Error},
  UsersResetPasswordRequest, UsersResetPasswordResponse, UsersResetPasswordResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName, EventParameterKey, EventStatus},
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
  tr,
};
use serde_json::json;
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::controller::{audit::process_audit, ApiController};
use crate::models::users::users_forgot_password::{
  users_forgot_password_validate, users_reset_password_auditable,
};

pub async fn users_reset_password(
  ctr: &ApiController,
  request: Request<UsersResetPasswordRequest>,
) -> Result<Response<UsersResetPasswordResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();
  let path = "api.users.users_reset_password";
  let req = request.into_inner();

  ctr.metrics.record_users_reset_password_success();

  let mut audit = AuditRecord::new(ctx.clone(), EventName::UsersResetPassword, EventStatus::Fail);

  let req_clone = req.clone();
  let audit_future = spawn(async move { users_reset_password_auditable(&req_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    audit_clone.set_event_parameter(EventParameterKey::Data, get_audit().await);
    process_audit(&audit_clone);
    Response::new(UsersResetPasswordResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = users_forgot_password_validate(ctx.clone(), path, &req) {
    ctr.metrics.record_users_reset_password_failure();
    return Ok(return_err(err).await);
  }

  // Get token from database
  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("tokens_get_by_token");

  let db_res = ctr.sql_db.tokens_get_by_token(ctx.clone(), &req.token).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("tokens_get_by_token", db_duration);

  let token = match db_res {
    Ok(token) => token,
    Err(err) => {
      ctr.metrics.record_db_error("tokens_get_by_token", &err.msg);
      ctr.metrics.record_users_reset_password_failure();
      let id = match err.err_type {
        ErrorType::NotFound => "users.reset_password.token_invalid",
        _ => ERROR_ID_INTERNAL,
      };
      let err_res = match err.err_type {
        ErrorType::NotFound => {
          AppError::new(ctx.clone(), path, id, None, "", Code::NotFound.into(), None)
        }
        _ => ie(Box::new(err)),
      };
      return Ok(return_err(err_res).await);
    }
  };

  if token.r#type.to_i32() != TokenType::PasswordReset.to_i32() {
    ctr.metrics.record_users_reset_password_failure();
    let id = "users.reset_password.token_invalid";
    let err_res = AppError::new(ctx.clone(), path, id, None, "", Code::NotFound.into(), None);
    return Ok(return_err(err_res).await);
  }

  if token.used {
    ctr.metrics.record_users_reset_password_failure();
    let id = "users.reset_password.token_invalid";
    let err_res = AppError::new(ctx.clone(), path, id, None, "", Code::NotFound.into(), None);
    return Ok(return_err(err_res).await);
  }

  let now = chaty_utils::time::time_get_seconds() as i64;
  if now > token.expires_at {
    ctr.metrics.record_users_reset_password_failure();
    let id = "users.reset_password.token_expired";
    let err_res =
      AppError::new(ctx.clone(), path, id, None, "", Code::DeadlineExceeded.into(), None);
    return Ok(return_err(err_res).await);
  }

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("users_get_by_id");

  let db_res = ctr.sql_db.users_get_by_id(ctx.clone(), &token.user_id).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("users_get_by_id", db_duration);

  let _user = match db_res {
    Ok(user) => user,
    Err(err) => {
      ctr.metrics.record_db_error("users_get_by_id", &err.msg);
      ctr.metrics.record_users_reset_password_failure();
      let err_res = ie(Box::new(err));
      return Ok(return_err(err_res).await);
    }
  };

  let salt = SaltString::generate(rand::thread_rng());
  let password_hash = match Argon2::default().hash_password(req.password.as_bytes(), &salt) {
    Ok(hash) => hash.to_string(),
    Err(err) => {
      ctr.metrics.record_users_reset_password_failure();
      let msg = format!("an error occurred when hashing a password: {}", err.to_string());
      return Ok(return_err(ie(Box::new(StdErr::new(ErrorKind::Other, msg)))).await);
    }
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("users_update_password");

  if let Err(err) =
    ctr.sql_db.users_update_password(ctx.clone(), &token.user_id, &password_hash).await
  {
    ctr.metrics.record_db_error("users_update_password", &err.msg);
    ctr.metrics.record_users_reset_password_failure();
    let err_res = ie(Box::new(err));
    return Ok(return_err(err_res).await);
  }

  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("users_update_password", db_duration);

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("tokens_mark_as_used");

  if let Err(err) = ctr.sql_db.tokens_mark_as_used(ctx.clone(), &token.id).await {
    tracing::error!("Failed to mark token as used: {:?}", err);
    ctr.metrics.record_db_error("tokens_mark_as_used", &err.msg);
    // Don't fail the request, password is already reset
  }

  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("tokens_mark_as_used", db_duration);

  audit.set_event_parameter(EventParameterKey::Data, get_audit().await);
  audit.success();
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("users.users_reset_password", request_duration);

  let message = tr::<()>(lang, "users.reset_password.success", None).unwrap_or_else(|_| {
    "Your password has been reset successfully. You can now log in with your new password."
      .to_string()
  });

  Ok(Response::new(UsersResetPasswordResponse {
    response: Some(Data(UsersResetPasswordResponseData { message })),
  }))
}
