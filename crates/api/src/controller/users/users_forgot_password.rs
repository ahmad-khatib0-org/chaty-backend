use std::sync::Arc;

use chaty_database::Token;
use chaty_proto::{
  users_forgot_password_response::Response::{Data, Error},
  UsersForgotPasswordRequest, UsersForgotPasswordResponse, UsersForgotPasswordResponseData,
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
use ulid::Ulid;

use crate::controller::{audit::process_audit, ApiController};

pub async fn users_forgot_password(
  ctr: &ApiController,
  request: Request<UsersForgotPasswordRequest>,
) -> Result<Response<UsersForgotPasswordResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let lang = ctx.accept_language();
  let path = "api.users.users_forgot_password";
  let req = request.into_inner();

  ctr.metrics.record_users_forgot_password_success();

  let mut audit = AuditRecord::new(ctx.clone(), EventName::UsersForgotPassword, EventStatus::Fail);

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    audit_clone.fail();
    process_audit(&audit_clone);
    Response::new(UsersForgotPasswordResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("users_get_by_email");

  let db_res = ctr.sql_db.users_get_by_email(ctx.clone(), &req.email).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("users_get_by_email", db_duration);

  let user = match db_res {
    Ok(user) => user,
    Err(err) => {
      ctr.metrics.record_db_error("users_get_by_email", &err.msg);
      ctr.metrics.record_users_forgot_password_failure();
      let id = "users.forgot_password.email_not_found";
      let err_res = match err.err_type {
        ErrorType::NotFound => {
          AppError::new(ctx.clone(), path, id, None, "", Code::NotFound.into(), None)
        }
        _ => ie(Box::new(err)),
      };
      return Ok(return_err(err_res).await);
    }
  };

  let now = time_get_seconds();
  let token = Token {
    id: Ulid::new().to_string(),
    user_id: user.id.to_string(),
    token: format!("reset_{}", Ulid::new()),
    r#type: chaty_database::TokenType::PasswordReset,
    used: false,
    created_at: now as i64,
    expires_at: (now + 86400) as i64, // 24 hours
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("tokens_create");

  if let Err(err) = ctr.sql_db.tokens_create(ctx.clone(), &token).await {
    tracing::error!("Failed to create password reset token: {:?}", err);
    ctr.metrics.record_db_error("tokens_create", &err.msg);
    ctr.metrics.record_users_forgot_password_failure();
    let message = json!({
      "user_id": user.id.to_string(),
      "email": user.email,
      "username": user.username
    });
    if let Err(dlq_err) = ctr.broker.clone().publish_password_reset_dlq(&message).await {
      tracing::error!("Failed to publish to DLQ: {:?}", dlq_err);
    }
    let err_to_return = ie(Box::new(err));
    return Ok(return_err(err_to_return).await);
  }

  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("tokens_create", db_duration);

  // Publish password reset message to broker
  let broker_start = std::time::Instant::now();
  let message = json!({
    "user_id": user.id.to_string(),
    "email": user.email,
    "username": user.username,
    "reset_token": token.token,
    "language": lang
  });

  if let Err(err) = ctr.broker.publish_password_reset(&message).await {
    tracing::error!("Failed to publish password reset message: {:?}", err);
    ctr.metrics.record_broker_message_failed();
    if let Err(dlq_err) = ctr.broker.publish_password_reset_dlq(&message).await {
      tracing::error!("Failed to publish to DLQ: {:?}", dlq_err);
    }
  } else {
    ctr.metrics.record_broker_message_sent();
  }

  let broker_duration = broker_start.elapsed().as_secs_f64();
  ctr.metrics.observe_broker_operation_duration("password_reset_publish", broker_duration);

  // Success
  audit.success();
  audit.set_event_parameter(EventParameterKey::UserId, json!(user.id));
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("users.users_forgot_password", request_duration);

  let message = tr::<()>(lang, "users.forgot_password.success", None)
    .unwrap_or_else(|_| "Password reset link has been sent to your email.".to_string());

  Ok(Response::new(UsersForgotPasswordResponse {
    response: Some(Data(UsersForgotPasswordResponseData { message })),
  }))
}
