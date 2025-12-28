use std::collections::HashMap;

use chaty_proto::{Timestamp, Value};
use serde::{Deserialize, Serialize};

/// Represents the standard JWT claims set (RFC 7519).
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct JwtClaims {
  /// "iss" (Issuer) - string or URI
  pub iss: String,
  /// "sub" (Subject)
  pub sub: String,
  /// "aud" (Audience) - could be a list
  pub aud: Vec<String>,
  /// "exp" (Expiration time)
  pub exp: Option<Timestamp>,
  /// "nbf" (Not before)
  pub nbf: Option<Timestamp>,
  /// "iat" (Issued at)
  pub iat: Option<Timestamp>,
  /// "jti" (JWT ID)
  pub jti: String,
  /// Custom claims (flexible map, e.g. roles, tenant_id, etc.)
  pub custom: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct CachedTokenStatus {
  pub dev_id: String,
  pub last_checked: i64,
  pub revoked: bool,
}

#[derive(Debug)]
pub struct EssentialHttpHeaders {
  pub path: String,
  pub method: String,
  pub user_agent: String,
  pub x_forwarded_for: String,
  pub x_request_id: String,
  pub accept_language: String,
  pub timezone: String,
  pub headers: std::collections::HashMap<String, String>,
}
