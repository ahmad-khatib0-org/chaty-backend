use std::collections::HashMap;

use chaty_proto::{envoy_service::auth::v3::CheckRequest, Timestamp};
use tonic::Request;

use crate::models::network::{EssentialHttpHeaders, JwtClaims};

pub fn extract_jwt_token_from_request<T>(req: &Request<T>) -> Option<String> {
  req.metadata().get("authorization")?.to_str().ok()?.strip_prefix("Bearer ")?.to_string().into()
}

pub fn extract_jwt_claims_and_token(req: &Request<CheckRequest>) -> (JwtClaims, String) {
  let parse_timestamp = |s: &str| -> Option<Timestamp> {
    s.parse::<i64>().ok().map(|secs| Timestamp { seconds: secs, nanos: 0 })
  };

  // build headers map anyway for extraction
  let mut headers_map: HashMap<String, String> = HashMap::new();
  if let Some(attrs) = req.get_ref().attributes.as_ref() {
    if let Some(r) = attrs.request.as_ref() {
      if let Some(http_req) = r.http.as_ref() {
        headers_map =
          http_req.headers.clone().into_iter().map(|(k, v)| (k.to_ascii_lowercase(), v)).collect();
      }
    }
  }

  let get =
    |k: &str| -> String { headers_map.get(&k.to_ascii_lowercase()).cloned().unwrap_or_default() };
  let get_ts = |k: &str| -> Option<Timestamp> {
    headers_map.get(&k.to_ascii_lowercase()).and_then(|s| parse_timestamp(s))
  };

  // Extract token from headers
  let mut token = headers_map
    .get("authorization")
    .and_then(|auth| auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer ")))
    .map(|s| s.to_string())
    .unwrap_or_default();

  // 4) Fallback: gRPC metadata
  let meta = req.metadata();

  if token.is_empty() {
    if let Some(auth_val) = meta.get("authorization") {
      if let Ok(auth_str) = auth_val.to_str() {
        token = auth_str
          .strip_prefix("Bearer ")
          .or_else(|| auth_str.strip_prefix("bearer "))
          .unwrap_or("")
          .to_string();
      }
    }
  }

  let claims = JwtClaims {
    iss: get("x-jwt-iss"),
    sub: get("x-jwt-sub"),
    aud: headers_map.get("x-jwt-aud").map(|s| vec![s.clone()]).unwrap_or_default(),
    exp: get_ts("x-jwt-exp"),
    nbf: get_ts("x-jwt-nbf"),
    iat: get_ts("x-jwt-iat"),
    jti: get("x-jwt-jti"),
    custom: Default::default(),
  };

  (claims, token)
}

pub fn get_essential_http_headers(
  req: &CheckRequest,
  languages: Vec<String>,
  default_language: String,
) -> EssentialHttpHeaders {
  let headers = req
    .attributes
    .as_ref()
    .and_then(|a| a.request.as_ref())
    .and_then(|r| r.http.as_ref())
    .map(|h| {
      h.headers.iter().map(|(k, v)| (k.to_lowercase(), v.clone())).collect::<HashMap<_, _>>()
    })
    .unwrap_or_default();

  let get = |key: &str| headers.get(key).cloned().unwrap_or_default();

  let existing_lang = get("accept-language");
  let accept_language = if languages.iter().any(|lang| lang == &existing_lang) {
    existing_lang
  } else {
    default_language
  };

  EssentialHttpHeaders {
    path: req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .unwrap_or_default(),

    method: req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.method.clone())
      .unwrap_or_default(),

    user_agent: get("user-agent"),
    x_forwarded_for: get("x-forwarded-for"),
    x_request_id: get("x-request-id"),
    accept_language,
    timezone: get("x-timezone"),
    headers,
  }
}
