use std::sync::Arc;

use chaty_proto::{
  config::core::v3::{header_value_option::HeaderAppendAction, HeaderValue, HeaderValueOption},
  envoy_service::auth::v3::{
    check_response::HttpResponse, CheckRequest, CheckResponse, OkHttpResponse,
  },
  google::{protobuf::BoolValue, rpc::Status},
};
use chaty_result::{
  context::{Context, Session},
  errors::{BoxedErr, ErrorType, InternalError},
  network::Header,
  tr,
};
use chrono::Utc;
use tonic::{Code, Request, Response};

use crate::{
  models::network::JwtClaims,
  utils::network::{extract_jwt_token_from_request, get_essential_http_headers},
};

use super::Controller;

impl Controller {
  // TODO: handle getting the real ip
  pub async fn get_context(&self, req: &CheckRequest) -> Arc<Context> {
    let h = get_essential_http_headers(
      req,
      self.cached_config.available_languages.clone(),
      self.cached_config.default_language.clone(),
    );

    Arc::new(Context {
      session: Session::default(),
      ip_address: h.x_forwarded_for.clone(),
      x_forwarded_for: h.x_forwarded_for,
      request_id: h.x_request_id,
      path: h.path,
      user_agent: h.user_agent,
      accept_language: h.accept_language,
      timezone: h.timezone,
    })
  }

  pub async fn response_ok(
    &self,
    ctx: &Arc<Context>,
    req: &Request<CheckRequest>,
    claims: Option<JwtClaims>,
  ) -> Response<CheckResponse> {
    let headers = self.prepare_headers(ctx, req, claims).await;
    if headers.is_err() {
      return Response::new(CheckResponse {
        status: Some(Status {
          code: Code::Internal as i32,
          message: Controller::int_err_msg(&ctx.accept_language),
          details: vec![],
        }),
        ..Default::default()
      });
    }

    Response::new(CheckResponse {
      status: Some(Status { code: Code::Ok as i32, message: "".into(), details: vec![] }),
      http_response: Some(HttpResponse::OkResponse(OkHttpResponse {
        headers: headers.unwrap(),
        ..Default::default()
      })),
      ..Default::default()
    })
  }

  pub async fn prepare_headers(
    &self,
    ctx: &Arc<Context>,
    req: &Request<CheckRequest>,
    claims: Option<JwtClaims>,
  ) -> Result<Vec<HeaderValueOption>, BoxedErr> {
    let mut headers: Vec<HeaderValueOption> = vec![];
    let device_id = "dump device id";

    let header = |header: Header, value: String| {
      HeaderValueOption {
        append: Some(BoolValue { value: false }),
        append_action: HeaderAppendAction::OverwriteIfExistsOrAdd.into(),
        keep_empty_value: false, // don't add header if value is empty,
        header: Some(HeaderValue {
          key: header.to_string(),
          value,
          raw_value: Vec::new(), // leave empty unless raw bytes is needed
        }),
      }
    };

    if claims.is_some() {
      let c = claims.unwrap().clone();
      if !c.jti.is_empty() && !c.sub.is_empty() {
        let token = extract_jwt_token_from_request(req).unwrap_or_default();
        let user_id = c.sub.clone();
        let auth_data =
          self.get_or_insert_auth_cached_user_data(ctx.clone(), &user_id).await.map_err(|err| {
            let msg =
              "failed to get/insert uesr data to be fowarded to downstream services as metadata";
            InternalError {
              err,
              err_type: ErrorType::InternalError,
              msg: msg.into(),
              temp: true,
              path: "auth.controller.prepare_headers".into(),
            }
          })?;

        headers.push(header(Header::XSessionID, c.jti));
        headers.push(header(Header::Authorization, token));
        headers.push(header(
          Header::XSessionCreatedAt,
          c.iat.and_then(|t| t.seconds.to_string().into()).unwrap_or_default(),
        ));
        headers.push(header(
          Header::XSessionExpiresAt,
          c.exp.and_then(|t| t.seconds.to_string().into()).unwrap_or_default(),
        ));
        headers.push(header(Header::XLastActivityAt, Utc::now().timestamp().to_string()));
        headers.push(header(Header::XUserID, user_id));
        headers.push(header(Header::XDeviceID, device_id.to_string()));
        headers.push(header(Header::XRoles, auth_data.roles));
        headers.push(header(Header::XIsOAuth, auth_data.is_oauth.to_string()));
        headers.push(header(Header::XProps, auth_data.props));
      }
    }

    headers.push(header(Header::XRequestID, ctx.request_id.clone()));
    headers.push(header(Header::XIPAddress, ctx.ip_address.clone()));
    headers.push(header(Header::XForwardedFor, ctx.x_forwarded_for.clone()));
    headers.push(header(Header::UserAgent, ctx.user_agent.clone()));
    headers.push(header(Header::AcceptLanguage, ctx.accept_language.clone()));
    headers.push(header(Header::XTimezone, ctx.timezone.clone()));
    Ok(headers)
  }

  pub fn not_found_msg(lang: &str) -> String {
    return tr::<String>(lang, "error.not_found", None)
      .unwrap_or("The requested path is not provided!".into());
  }

  pub fn invalid_token_msg(lang: &str) -> String {
    tr::<String>(lang, "jwt.payload.invalid", None)
      .unwrap_or("Sorry, the authentication payload is invalid, please login first".into())
  }

  pub fn int_err_msg(lang: &str) -> String {
    return tr::<String>(lang, "error.internal", None).unwrap_or(
      "Sorry, Unexpected internal server error. Our team has been notified. Please try again"
        .into(),
    );
  }
}

pub trait CheckResponseExt {
  fn denied(msg: &str) -> Self;
}

impl CheckResponseExt for CheckResponse {
  fn denied(msg: &str) -> Self {
    let header = |key: &str, value: String| HeaderValueOption {
      append: Some(BoolValue { value: false }),
      append_action: HeaderAppendAction::OverwriteIfExistsOrAdd as i32,
      keep_empty_value: false,
      header: Some(HeaderValue { key: key.to_string(), value, raw_value: Vec::new() }),
    };

    Self {
      status: Some(Status {
        code: Code::PermissionDenied as i32,
        message: msg.to_string(),
        details: vec![],
      }),
      http_response: Some(HttpResponse::OkResponse(OkHttpResponse {
        headers: vec![
          header("x-grpc-message", msg.to_string()),
          header("x-grpc-status", (Code::PermissionDenied as i32).to_string()),
          header("x-error-message", msg.to_string()), // Additional header
        ],
        ..Default::default()
      })),
      ..Default::default()
    }
  }
}
