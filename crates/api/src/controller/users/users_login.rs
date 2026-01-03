use std::{
  collections::HashMap,
  io::{Error as StdErr, ErrorKind},
  sync::Arc,
};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chaty_proto::{
  users_login_response::Response::{Data, Error},
  UsersLoginRequest, UsersLoginResponse, UsersLoginResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName, EventParameterKey, EventStatus},
  context::Context,
  errors::{AppError, AppErrorError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
};
use serde_json::json;
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::{
  controller::{audit::process_audit, ApiController},
  models::users::users_login::{
    get_oauth_request_err_msg_id, users_login_auditable, users_login_validate, OAuthAcceptResult,
    OAuthErrorResponse,
  },
};

pub async fn users_login(
  ctr: &ApiController,
  request: Request<UsersLoginRequest>,
) -> Result<Response<UsersLoginResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.users.users_login";
  let req = request.into_inner();

  let mut audit = AuditRecord::new(ctx.clone(), EventName::UsersLogin, EventStatus::Fail);

  let login_clone = req.clone();
  let audit_future = spawn(async move { users_login_auditable(&login_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    let data = get_audit().await;
    audit_clone.set_event_parameter(EventParameterKey::UsersLogin, data);
    process_audit(&audit_clone);
    Response::new(UsersLoginResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    println!("an error {}", err);
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = users_login_validate(ctx.clone(), &path, &req) {
    return Ok(return_err(err).await);
  }

  let db_res = ctr.sql_db.clone().users_get_by_email(ctx.clone(), &req.email).await;
  if db_res.is_err() {
    let err = db_res.unwrap_err();
    match err.err_type {
      ErrorType::NotFound => {
        let e = ("users.email.not_found", Code::NotFound); // just to prevent many lines
        return Ok(return_err(AppError::new(ctx, path, e.0, None, "", e.1.into(), None)).await);
      }
      _ => return Ok(return_err(ie(Box::new(err))).await),
    }
  }
  let user = db_res.unwrap();

  let parsed_hash = PasswordHash::new(&user.password);
  if parsed_hash.is_err() {
    let msg = parsed_hash.unwrap_err().to_string();
    return Ok(return_err(ie(Box::new(StdErr::new(ErrorKind::Other, msg)))).await);
  }

  let is_valid =
    Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash.unwrap()).is_ok();
  if !is_valid {
    let e = ("users.credentials.error", Code::InvalidArgument); // just to prevent many lines
    return Ok(return_err(AppError::new(ctx, path, e.0, None, "", e.1.into(), None)).await);
  }

  let client = ctr.http_client.clone();
  let base = ctr.config.clone().oauth.admin_url.clone();

  let payload = json!({
    "subject": user.id,
    "remember": true,
    "remember_for": 240 * 60 * 60,
    "context": {
        "lang": ctx.accept_language(),
        "email": user.email,
    }
  });
  let response = client
    .put(format!(
      "{}/oauth2/auth/requests/login/accept?login_challenge={}",
      base, req.login_challenge
    ))
    .header("Content-Type", "application/json")
    .json(&payload)
    .send()
    .await;
  if response.is_err() {
    println!("send request error");
    return Ok(return_err(ie(Box::new(response.unwrap_err()))).await);
  }

  let response = response.unwrap();
  let status = response.status();

  if !status.is_success() {
    println!("is_success is false");
    println!("Status: {}, URL: {}", status, response.url());

    let bytes = response.bytes().await;
    if bytes.is_err() {
      println!("decoding resposne to bytes failed");
      return Ok(return_err(ie(Box::new(bytes.unwrap_err()))).await);
    }
    let bytes = bytes.unwrap();

    let response_text = String::from_utf8_lossy(&bytes).to_string();
    println!("Response body: {}", response_text);

    let err_res = match serde_json::from_slice::<OAuthErrorResponse>(&bytes) {
      Ok(err) => err,
      Err(err) => {
        return Ok(return_err(ie(Box::new(err))).await);
      }
    };

    let id = get_oauth_request_err_msg_id(&err_res.error, &err_res.error_description);
    let errors = AppErrorErrors {
      err: Some(Box::new(StdErr::new(ErrorKind::Other, err_res.to_string()))),
      errors_internal: Some(HashMap::from([
        ("error".to_string(), AppErrorError { id: "users.login.error".to_string(), params: None }),
        ("error_description".to_string(), AppErrorError { id, params: None }),
      ])),
      ..Default::default()
    };

    let error_kind = if status.is_client_error() { Code::InvalidArgument } else { Code::Internal };
    let e = ("users.login.error", Some(errors));
    return Ok(return_err(AppError::new(ctx, path, e.0, None, "", error_kind.into(), e.1)).await);
  }

  let result = response.json::<OAuthAcceptResult>().await;
  if result.is_err() {
    println!("decode result error");
    return Ok(return_err(ie(Box::new(result.unwrap_err()))).await);
  }
  let result = result.unwrap();
  if result.redirect_to.is_empty() {
    let msg = "received an empty redirect_url from OAuth service login/accept";
    return Ok(return_err(ie(Box::new(StdErr::new(ErrorKind::Other, msg)))).await);
  }

  let data = get_audit().await;
  audit.set_event_parameter(EventParameterKey::UsersCreate, data);
  audit.success();
  process_audit(&audit);

  let cleaned_redirect = result
    .redirect_to
    .replace("prompt=login&", "")
    .replace("&prompt=login", "")
    .replace("prompt=login", "");

  println!("the redirect to {}", cleaned_redirect);
  Ok(Response::new(UsersLoginResponse {
    response: Some(Data(UsersLoginResponseData { redirect_to: cleaned_redirect })),
  }))
}
