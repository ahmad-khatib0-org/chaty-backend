use std::{collections::HashMap, fmt, sync::Arc};

use chaty_proto::UsersLoginRequest;
use chaty_result::{
  context::Context,
  errors::{AppError, OptionalParams},
  tr,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tonic::Code;
use validator::ValidateEmail;

use crate::models::users::users_create::{USERS_PASSWORD_MAX_LENGTH, USERS_PASSWORD_MIN_LENGTH};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthErrorResponse {
  pub error: String,
  #[serde(rename = "error_description")]
  pub error_description: String,
  #[serde(rename = "error_hint", skip_serializing_if = "Option::is_none")]
  pub error_hint: Option<String>,
  #[serde(rename = "error_debug", skip_serializing_if = "Option::is_none")]
  pub error_debug: Option<String>,
}

impl fmt::Display for OAuthErrorResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.error, self.error_description)?;
    if let Some(debug) = &self.error_debug {
      write!(f, " (debug: {})", debug)?;
    }
    if let Some(hint) = &self.error_hint {
      write!(f, " [hint: {}]", hint)?;
    }
    Ok(())
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthAcceptResult {
  #[serde(rename = "redirect_to")]
  pub redirect_to: String,
}

pub fn users_login_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &UsersLoginRequest,
) -> Result<(), AppError> {
  let ae = |id: &str, params: OptionalParams| {
    return AppError::new(ctx.clone(), path, id, params, "", Code::InvalidArgument.into(), None);
  };

  if req.email.trim().is_empty() {
    return Err(ae("users.email.required", None));
  }
  if !req.email.validate_email() {
    return Err(ae("users.email.invalid", None));
  }

  if req.password.trim().is_empty() {
    return Err(ae("users.password.required", None));
  }
  if req.password.len() < USERS_PASSWORD_MIN_LENGTH
    || req.password.len() > USERS_PASSWORD_MAX_LENGTH
  {
    return Err(ae(
      "users.password.length",
      Some(HashMap::from([
        ("Min".to_string(), Value::Number(USERS_PASSWORD_MIN_LENGTH.into())),
        ("Max".to_string(), Value::Number(USERS_PASSWORD_MAX_LENGTH.into())),
      ])),
    ));
  }

  Ok(())
}

// create an auditable request to be saved
pub fn users_login_auditable(user: &UsersLoginRequest) -> Value {
  json!({ "email": user.email })
}

pub fn get_oauth_request_err_msg(lang: &str, code: &str, desc: &str) -> String {
  let tr = |id: &str| -> String {
    tr(lang, id, None::<()>)
            .unwrap_or_else(|_| "Invalid authentication configuration. Try again, if issue persists, Please contact support.".to_string())
  };

  match code {
    "invalid_request" => {
      if desc.contains("redirect_uri") {
        tr("oauth.invalid_request.redirect_uri")
      } else {
        tr("oauth.invalid_request.general")
      }
    }
    "access_denied" => tr("oauth.access_denied.user"),
    "unauthorized_client" => tr("oauth.unauthorized_client"),
    "unsupported_response_type" => tr("oauth.unsupported_response_type"),
    "invalid_scope" => tr("oauth.invalid_scope"),
    "server_error" => tr("oauth.server_error.internal"),
    "temporarily_unavailable" => tr("oauth.temporarily_unavailable"),
    _ => tr("oauth.unknown_error"),
  }
}

pub fn get_oauth_request_err_msg_id(code: &str, desc: &str) -> String {
  match code {
    "invalid_request" => {
      if desc.contains("redirect_uri") {
        "oauth.invalid_request.redirect_uri".to_string()
      } else {
        "oauth.invalid_request.general".to_string()
      }
    }
    "access_denied" => "oauth.access_denied.user".to_string(),
    "unauthorized_client" => "oauth.unauthorized_client".to_string(),
    "unsupported_response_type" => "oauth.unsupported_response_type".to_string(),
    "invalid_scope" => "oauth.invalid_scope".to_string(),
    "server_error" => "oauth.server_error.internal".to_string(),
    "temporarily_unavailable" => "oauth.temporarily_unavailable".to_string(),
    _ => "oauth.unknown_error".to_string(),
  }
}
