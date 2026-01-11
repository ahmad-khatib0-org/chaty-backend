use std::{str::FromStr, sync::Arc};

use chaty_proto::{PaginationRequest, PaginationResponse};
use derive_more::Display;
use tonic::Code;
use ulid::Ulid;

use crate::{context::Context, errors::AppError};

pub fn check_last_id(
  ctx: Arc<Context>,
  _where: &str,
  pagination: &Option<PaginationRequest>,
) -> Result<(), AppError> {
  let mk_err =
    |id: &str| AppError::new(ctx, _where, id, None, "", Code::InvalidArgument.into(), None);

  if pagination.is_none() {
    return Err(mk_err("request.pagination.invalid"));
  }

  let pagination = pagination.as_ref().unwrap();
  let last_id = pagination.last_id();
  let page = pagination.page();

  if page > 1 && last_id == "" {
    return Err(mk_err("request.last_id.missing"));
  }
  if page > 1 {
    Ulid::from_string(last_id).map_err(|_| mk_err("request.last_id.invalid"))?;
  }
  Ok(())
}

pub fn build_pagination_response(pr: &PaginationRequest, items_count: usize) -> PaginationResponse {
  PaginationResponse {
    has_previous: Some(pr.page() != 1),
    has_next: Some(pr.page_size() == items_count as i32),
    ..Default::default()
  }
}

pub fn is_valid_ulid(id: &str) -> bool {
  if id.len() != 26 {
    return false;
  }
  let result = Ulid::from_str(id);
  if result.is_err() {
    return false;
  }
  true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Header {
  #[display("authorization")]
  Authorization,
  #[display("x-request-id")]
  XRequestID,
  #[display("x-correlation-id")]
  XCorrelationID,
  #[display("x-ip-address")]
  XIPAddress,
  #[display("x-forwarded-for")]
  XForwardedFor,
  #[display("x-forwarded-proto")]
  XForwardedProto,
  #[display("x-forwarded-host")]
  XForwardedHost,
  #[display("x-client-version")]
  XClientVersion,
  #[display("x-client-id")]
  XClientID,
  #[display("x-device-id")]
  XDeviceID,
  #[display("x-session-id")]
  XSessionID,
  #[display("x-user-id")]
  XUserID,
  #[display("x-trace-id")]
  XTraceID,
  #[display("x-span-id")]
  XSpanID,
  #[display("x-roles")]
  XRoles,
  #[display("x-is-oauth")]
  XIsOAuth,
  #[display("x-session-created-at")]
  XSessionCreatedAt,
  #[display("x-session-expires-at")]
  XSessionExpiresAt,
  #[display("x-last-activity-at")]
  XLastActivityAt,
  #[display("x-timezone")]
  XTimezone,
  #[display("x-props")]
  XProps,
  #[display("x-api-key")]
  XAPIKey,
  #[display("x-csrf-token")]
  XCSRFToken,
  #[display("x-rate-limit-limit")]
  XRateLimitLimit,
  #[display("x-rate-limit-remaining")]
  XRateLimitRemaining,
  #[display("x-rate-limit-reset")]
  XRateLimitReset,
  // Standard headers
  #[display("content-type")]
  ContentType,
  #[display("user-agent")]
  UserAgent,
  #[display("accept")]
  Accept,
  #[display("accept-language")]
  AcceptLanguage,
  #[display("accept-encoding")]
  AcceptEncoding,
  #[display("cache-control")]
  CacheControl,
  // gRPC specific
  #[display("x-grpc-web")]
  GRPCWeb,
  #[display("grpc-encoding")]
  GRPCEncoding,
  #[display("grpc-message")]
  GRPCMessage,
  #[display("grpc-status")]
  GRPCStatus,
}

impl Header {
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Authorization => "authorization",
      Self::XRequestID => "x-request-id",
      Self::XCorrelationID => "x-correlation-id",
      Self::XIPAddress => "x-ip-address",
      Self::XForwardedFor => "x-forwarded-for",
      Self::XForwardedProto => "x-forwarded-proto",
      Self::XForwardedHost => "x-forwarded-host",
      Self::XClientVersion => "x-client-version",
      Self::XClientID => "x-client-id",
      Self::XDeviceID => "x-device-id",
      Self::XSessionID => "x-session-id",
      Self::XUserID => "x-user-id",
      Self::XTraceID => "x-trace-id",
      Self::XSpanID => "x-span-id",
      Self::XRoles => "x-roles",
      Self::XIsOAuth => "x-is-oauth",
      Self::XSessionCreatedAt => "x-session-created-at",
      Self::XSessionExpiresAt => "x-session-expires-at",
      Self::XLastActivityAt => "x-last-activity-at",
      Self::XTimezone => "x-timezone",
      Self::XProps => "x-props",
      Self::XAPIKey => "x-api-key",
      Self::XCSRFToken => "x-csrf-token",
      Self::XRateLimitLimit => "x-rate-limit-limit",
      Self::XRateLimitRemaining => "x-rate-limit-remaining",
      Self::XRateLimitReset => "x-rate-limit-reset",
      Self::ContentType => "content-type",
      Self::UserAgent => "user-agent",
      Self::Accept => "accept",
      Self::AcceptLanguage => "accept-language",
      Self::AcceptEncoding => "accept-encoding",
      Self::CacheControl => "cache-control",
      Self::GRPCWeb => "x-grpc-web",
      Self::GRPCEncoding => "grpc-encoding",
      Self::GRPCMessage => "grpc-message",
      Self::GRPCStatus => "grpc-status",
    }
  }
}
