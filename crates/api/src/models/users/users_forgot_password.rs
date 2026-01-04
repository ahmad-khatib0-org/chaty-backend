use std::{collections::HashMap, sync::Arc};

use chaty_proto::UsersResetPasswordRequest;
use chaty_result::{
  context::Context,
  errors::{AppError, OptionalParams},
};
use serde_json::{json, Value};
use tonic::Code;

use crate::models::users::users_create::{USERS_PASSWORD_MAX_LENGTH, USERS_PASSWORD_MIN_LENGTH};

pub fn users_forgot_password_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &UsersResetPasswordRequest,
) -> Result<(), AppError> {
  let ae = |id: &str, params: OptionalParams| {
    return AppError::new(ctx.clone(), path, id, params, "", Code::InvalidArgument.into(), None);
  };

  if req.password != req.password_confirmation {
    return Err(ae("users.reset_password.password_mismatch", None));
  }

  if req.password.trim().len() > USERS_PASSWORD_MAX_LENGTH
    || req.password.trim().len() < USERS_PASSWORD_MIN_LENGTH
  {
    return Err(ae(
      "users.password.length",
      Some(HashMap::from([
        ("Min".to_string(), Value::Number(USERS_PASSWORD_MIN_LENGTH.into())),
        ("Max".to_string(), Value::Number(USERS_PASSWORD_MAX_LENGTH.into())),
      ])),
    ));
  }
  if !req.password.chars().any(|c| c.is_uppercase()) {
    return Err(ae("users.password.requires_uppercase", None));
  }
  if !req.password.chars().any(|c| c.is_lowercase()) {
    return Err(ae("users.password.requires_lowercase", None));
  }
  if !req.password.chars().any(|c| c.is_numeric()) {
    return Err(ae("users.password.requires_number", None));
  }
  if !req.password.chars().any(|c| !c.is_alphanumeric()) {
    return Err(ae("users.password.requires_symbol", None));
  }

  Ok(())
}

pub fn users_reset_password_auditable(req: &UsersResetPasswordRequest) -> serde_json::Value {
  json!({ "token": req.token })
}
