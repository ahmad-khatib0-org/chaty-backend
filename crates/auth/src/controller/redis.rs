use std::io::{Error, ErrorKind};
use std::sync::Arc;

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::{Connection, Pool};
use tonic::async_trait;
use tower::BoxError;
use tracing::instrument;

use crate::models::network::CachedTokenStatus;

use super::metrics::MetricsCollector;
use super::token::{check_token, get_token, mark_checked_ok, revoke_token, set_token};

/// Represents Redis check results
pub enum RedisCheck {
  Allowed { status: Option<CachedTokenStatus> },
  Revoked(String), // reason
}

#[async_trait]
pub trait RedisClient: Send + Sync {
  async fn check_token(&self, token: &str) -> Result<RedisCheck, BoxedErr>;
  async fn revoke_token(&self, token: &str) -> Result<(), BoxedErr>;
  async fn mark_checked_ok(&self, token: &str) -> Result<(), BoxedErr>;
  async fn get_token(&self, token: &str, path: &str)
    -> Result<Option<CachedTokenStatus>, BoxedErr>;
  async fn set_token(
    &self,
    jti: &str,
    data: &CachedTokenStatus,
    path: &str,
  ) -> Result<(), BoxedErr>;
}

/// Concrete Redis client wrapper
#[derive(Debug, Clone)]
pub struct DefaultRedisClient {
  pub redis: Arc<Pool>,
  pub metrics: MetricsCollector,
}

impl DefaultRedisClient {
  pub fn not_found_err(path: &str, jti: &str) -> BoxError {
    let msg = format!("redis key not found: {}", jti);
    let err = Box::new(Error::new(ErrorKind::NotFound, msg.clone()));
    Box::new(InternalError::new(path.into(), err, ErrorType::InternalError, false, msg))
  }

  #[instrument(skip(self))]
  pub async fn get_conn(&self, path: &str) -> Result<Connection, BoxedErr> {
    self.redis.get().await.map_err(|err| {
      let error_msg = err.to_string();
      self.metrics.record_redis_error("get_connection", &error_msg);
      let internal_err = InternalError::new(
        path.into(),
        Box::new(err),
        ErrorType::InternalError,
        false,
        "failed to get a redis connection from pool".into(),
      );
      Box::new(internal_err) as BoxedErr
    })
  }
}

#[async_trait]
impl RedisClient for DefaultRedisClient {
  #[instrument(skip(self), fields(token_id = ""))]
  async fn get_token(
    &self,
    token: &str,
    path: &str,
  ) -> Result<Option<CachedTokenStatus>, BoxedErr> {
    let start = std::time::Instant::now();
    tracing::Span::current().record("token_id", token);
    let result = get_token(self, &token, &path).await;
    let duration = start.elapsed().as_secs_f64();

    match &result {
      Ok(Some(_)) => {
        self.metrics.record_cache_hit();
        self.metrics.record_redis_operation("get_token");
      }
      Ok(None) => {
        self.metrics.record_cache_miss();
        self.metrics.record_redis_operation("get_token");
      }
      Err(e) => {
        self.metrics.record_redis_error("get_token", &e.to_string());
      }
    }
    self.metrics.observe_redis_duration("get_token", duration);
    result
  }

  #[instrument(skip(self, data), fields(token_id = ""))]
  async fn set_token(
    &self,
    jti: &str,
    data: &CachedTokenStatus,
    path: &str,
  ) -> Result<(), BoxedErr> {
    let start = std::time::Instant::now();
    tracing::Span::current().record("token_id", jti);
    let result = set_token(self, jti, data, path).await;
    let duration = start.elapsed().as_secs_f64();

    if let Err(e) = &result {
      self.metrics.record_redis_error("set_token", &e.to_string());
    } else {
      self.metrics.record_redis_operation("set_token");
    }
    self.metrics.observe_redis_duration("set_token", duration);
    result
  }

  #[instrument(skip(self), fields(token_id = ""))]
  async fn check_token(&self, jti: &str) -> Result<RedisCheck, BoxedErr> {
    let start = std::time::Instant::now();
    tracing::Span::current().record("token_id", jti);
    let result = check_token(&self, &jti).await;
    let duration = start.elapsed().as_secs_f64();

    if let Err(e) = &result {
      self.metrics.record_redis_error("check_token", &e.to_string());
    } else {
      self.metrics.record_redis_operation("check_token");
    }
    self.metrics.observe_redis_duration("check_token", duration);
    result
  }

  #[instrument(skip(self), fields(token_id = ""))]
  async fn revoke_token(&self, jti: &str) -> Result<(), BoxedErr> {
    let start = std::time::Instant::now();
    tracing::Span::current().record("token_id", jti);
    let result = revoke_token(&self, &jti).await;
    let duration = start.elapsed().as_secs_f64();

    if let Err(e) = &result {
      self.metrics.record_redis_error("revoke_token", &e.to_string());
    } else {
      self.metrics.record_redis_operation("revoke_token");
    }
    self.metrics.observe_redis_duration("revoke_token", duration);
    result
  }

  #[instrument(skip(self), fields(token_id = ""))]
  async fn mark_checked_ok(&self, jti: &str) -> Result<(), BoxedErr> {
    let start = std::time::Instant::now();
    tracing::Span::current().record("token_id", jti);
    let result = mark_checked_ok(&self, &jti).await;
    let duration = start.elapsed().as_secs_f64();

    if let Err(e) = &result {
      self.metrics.record_redis_error("mark_checked_ok", &e.to_string());
    } else {
      self.metrics.record_redis_operation("mark_checked_ok");
    }
    self.metrics.observe_redis_duration("mark_checked_ok", duration);
    result
  }
}
