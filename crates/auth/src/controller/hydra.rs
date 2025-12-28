use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use derive_more::Display;
use reqwest::Client;
use serde::Deserialize;
use tonic::async_trait;

/// Represents the result of a Hydra token validation.
#[derive(Debug)]
pub enum HydraValidation {
  Valid { sub: String, exp: i64 },
  Invalid(String), // reason why token is invalid
}

/// Trait for Hydra client behavior
#[async_trait]
pub trait HydraClient: Send + Sync {
  async fn validate_token(&self, token: &str) -> Result<HydraValidation, BoxedErr>;
}

/// Concrete Hydra client
#[derive(Debug)]
pub struct DefaultHydraClient {
  pub hydra_url: String,
  pub client_id: String,
  pub client_secret: String,
  pub http: Arc<Client>,
}

#[derive(Debug, Deserialize, Display)]
#[display(
    "IntrospectionResponse: active: {active}, sub: {sub}, exp: {exp}",
    sub = sub.as_deref().unwrap_or("None"),
    exp = exp.map(|e| e.to_string()).as_deref().unwrap_or("None"))
]
struct IntrospectionResponse {
  active: bool,
  sub: Option<String>,
  exp: Option<i64>,
}

#[async_trait]
impl HydraClient for DefaultHydraClient {
  async fn validate_token(&self, token: &str) -> Result<HydraValidation, BoxedErr> {
    let url = format!("{}/oauth2/introspect", self.hydra_url);
    let err_msg = "failed to request hydra client";
    let ie = |err: BoxedErr, msg: &str| {
      let path = "auth.controller.validate_token".to_string();
      Box::new(InternalError::new(path, err, ErrorType::InternalError, false, msg.into()))
    };

    // send request
    let resp = self
      .http
      .post(&url)
      .basic_auth(&self.client_id, Some(&self.client_secret))
      .form(&[("token", token)])
      .send()
      .await
      .map_err(|err| {
        tracing::error!(%url, "hydra introspect request failed: {:?}", err);
        ie(Box::new(err), err_msg)
      })?;

    // Log status
    tracing::debug!(status = %resp.status(), "hydra introspect status");

    let status = resp.status();
    // Read the body text to inspect error payloads (do this before attempting .json())
    let body_text = resp.text().await.map_err(|err| {
      tracing::error!("failed to read hydra response body: {:?}", err);
      ie(Box::new(err), "failed to read hydra response body")
    })?;
    tracing::debug!(body = %body_text, "hydra introspect body");

    // Handle non-2xx explicitly with logged body
    if !status.is_success() {
      tracing::warn!(status = %status, body = %body_text, "hydra returned non-success");
      let err = Error::new(ErrorKind::Other, format!("hydra returned {}", status));
      return Err(ie(Box::new(err), err_msg));
    }

    // parse JSON now that we have the body text
    let body: IntrospectionResponse = serde_json::from_str(&body_text).map_err(|err| {
      tracing::error!("failed to parse introspection JSON: {:?}", err);
      ie(Box::new(err), "failed to serialize IntrospectionResponse hydra response")
    })?;

    if body.active {
      Ok(HydraValidation::Valid { sub: body.sub.unwrap_or_default(), exp: body.exp.unwrap_or(0) })
    } else {
      Ok(HydraValidation::Invalid(format!("the token is invalid: {}", body).into()))
    }
  }
}
