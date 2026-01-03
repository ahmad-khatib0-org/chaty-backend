use std::sync::Arc;

use chaty_proto::{
  users_email_confirmation_response::Response::{Data, Error},
  UsersEmailConfirmationRequest, UsersEmailConfirmationResponse,
  UsersEmailConfirmationResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName, EventParameterKey, EventStatus},
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
  tr,
};
use chaty_utils::time::time_get_seconds;
use serde_json::json;
use tonic::{Code, Request, Response, Status};
use urlencoding::decode;

use crate::controller::{audit::process_audit, ApiController};

pub async fn users_email_confirmation(
  ctr: &ApiController,
  request: Request<UsersEmailConfirmationRequest>,
) -> Result<Response<UsersEmailConfirmationResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();
  let path = "api.users.users_email_confirmation";
  let req = request.into_inner();

  ctr.metrics.record_users_email_confirmation_success();

  let mut audit =
    AuditRecord::new(ctx.clone(), EventName::UsersEmailConfirmation, EventStatus::Fail);

  let audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    process_audit(&audit_clone);
    Response::new(UsersEmailConfirmationResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let decoded_token = decode(&req.token).map_err(|_| {});
  if decoded_token.is_err() {
    ctr.metrics.record_users_email_confirmation_failure();
    let id = "users.email_confirmation.token_invalid";
    let err = AppError::new(ctx.clone(), path, id, None, "", Code::InvalidArgument.into(), None);
    return Ok(return_err(err).await);
  }
  let decoded_token = decoded_token.unwrap();

  if decoded_token.is_empty() {
    ctr.metrics.record_users_email_confirmation_failure();
    let id = "users.email_confirmation.token_invalid";
    let err = AppError::new(ctx.clone(), path, id, None, "", Code::InvalidArgument.into(), None);
    return Ok(return_err(err).await);
  }

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("tokens_get_by_token");

  let db_res = ctr.sql_db.tokens_get_by_token(ctx.clone(), &decoded_token).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("tokens_get_by_token", db_duration);

  let token = match db_res {
    Ok(token) => token,
    Err(err) => {
      ctr.metrics.record_db_error("tokens_get_by_token", &err.msg);
      ctr.metrics.record_users_email_confirmation_failure();
      let id = "users.email_confirmation.token_invalid";
      let err_res = match err.err_type {
        ErrorType::NotFound => {
          AppError::new(ctx.clone(), path, id, None, "", Code::NotFound.into(), None)
        }
        _ => ie(Box::new(err)),
      };
      return Ok(return_err(err_res).await);
    }
  };

  let now = time_get_seconds() as i64;
  if now > token.expires_at {
    ctr.metrics.record_users_email_confirmation_failure();
    let id = "users.email_confirmation.token_expired";
    let err = AppError::new(ctx.clone(), path, id, None, "", Code::FailedPrecondition.into(), None);
    return Ok(return_err(err).await);
  }

  // Check if token already used
  if token.used {
    let message = tr::<()>(lang, "users.email_confirmation.already_confirmed", None)
      .unwrap_or_else(|_| "Your email has already been confirmed.".to_string());
    let data = UsersEmailConfirmationResponseData { message };
    audit.success();
    audit.set_event_parameter(EventParameterKey::UserId, json!(token.user_id));
    process_audit(&audit);
    return Ok(Response::new(UsersEmailConfirmationResponse { response: Some(Data(data)) }));
  }

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("tokens_mark_as_used");

  let db_res = ctr.sql_db.tokens_mark_as_used(ctx.clone(), &token.id).await;
  let db_duration = db_start.elapsed().as_secs_f64();

  ctr.metrics.observe_db_operation_duration("tokens_mark_as_used", db_duration);
  if let Err(err) = db_res {
    ctr.metrics.record_db_error("tokens_mark_as_used", &err.msg);
    ctr.metrics.record_users_email_confirmation_failure();
    return Ok(return_err(ie(Box::new(err))).await);
  }

  // Success
  audit.success();
  audit.set_event_parameter(EventParameterKey::UserId, json!(token.user_id));
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("users.users_email_confirmation", request_duration);

  let message = tr::<()>(lang, "users.email_confirmation.success", None)
    .unwrap_or_else(|_| "Your email has been confirmed successfully.".to_string());

  Ok(Response::new(UsersEmailConfirmationResponse {
    response: Some(Data(UsersEmailConfirmationResponseData { message })),
  }))
}
