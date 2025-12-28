use prometheus::{CounterVec, HistogramOpts, HistogramVec, IntCounter, Registry};

/// Prometheus metrics collector for auth service
#[derive(Clone, Debug)]
pub struct MetricsCollector {
  pub token_checks_total: IntCounter,
  pub token_checks_failed: IntCounter,
  pub tokens_revoked: IntCounter,
  pub authorization_allowed: IntCounter,
  pub authorization_denied: CounterVec,
  pub redis_operations_total: CounterVec,
  pub redis_operations_failed: CounterVec,
  pub cache_hits: IntCounter,
  pub cache_misses: IntCounter,
  pub hydra_validations_total: IntCounter,
  pub hydra_validations_failed: IntCounter,
  pub request_duration_seconds: HistogramVec,
  pub redis_operation_duration_seconds: HistogramVec,
}

impl MetricsCollector {
  pub fn new(registry: &Registry) -> Result<Self, String> {
    let token_checks_total = IntCounter::new("auth_token_checks_total", "Total token checks")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(token_checks_total.clone())).map_err(|e| e.to_string())?;

    let token_checks_failed =
      IntCounter::new("auth_token_checks_failed_total", "Total failed token checks")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(token_checks_failed.clone())).map_err(|e| e.to_string())?;

    let tokens_revoked = IntCounter::new("auth_tokens_revoked_total", "Total tokens revoked")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(tokens_revoked.clone())).map_err(|e| e.to_string())?;

    let authorization_allowed =
      IntCounter::new("auth_authorization_allowed_total", "Total successful authorizations")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(authorization_allowed.clone())).map_err(|e| e.to_string())?;

    let authorization_denied = CounterVec::new(
      prometheus::Opts::new("auth_authorization_denied_total", "Total denied authorizations"),
      &["reason"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(authorization_denied.clone())).map_err(|e| e.to_string())?;

    let redis_operations_total = CounterVec::new(
      prometheus::Opts::new("auth_redis_operations_total", "Total Redis operations"),
      &["operation"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(redis_operations_total.clone())).map_err(|e| e.to_string())?;

    let redis_operations_failed = CounterVec::new(
      prometheus::Opts::new("auth_redis_operations_failed_total", "Total failed Redis operations"),
      &["operation", "error"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(redis_operations_failed.clone())).map_err(|e| e.to_string())?;

    let cache_hits =
      IntCounter::new("auth_cache_hits_total", "Total cache hits").map_err(|e| e.to_string())?;
    registry.register(Box::new(cache_hits.clone())).map_err(|e| e.to_string())?;

    let cache_misses = IntCounter::new("auth_cache_misses_total", "Total cache misses")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(cache_misses.clone())).map_err(|e| e.to_string())?;

    let hydra_validations_total =
      IntCounter::new("auth_hydra_validations_total", "Total Hydra validations")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(hydra_validations_total.clone())).map_err(|e| e.to_string())?;

    let hydra_validations_failed =
      IntCounter::new("auth_hydra_validations_failed_total", "Total failed Hydra validations")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(hydra_validations_failed.clone())).map_err(|e| e.to_string())?;

    let request_duration_seconds = HistogramVec::new(
      HistogramOpts::new("auth_request_duration_seconds", "Request duration in seconds"),
      &[],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(request_duration_seconds.clone())).map_err(|e| e.to_string())?;

    let redis_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("auth_redis_operation_duration_seconds", "Redis operation duration"),
      &["operation"],
    )
    .map_err(|e| e.to_string())?;
    registry
      .register(Box::new(redis_operation_duration_seconds.clone()))
      .map_err(|e| e.to_string())?;

    Ok(MetricsCollector {
      token_checks_total,
      token_checks_failed,
      tokens_revoked,
      authorization_allowed,
      authorization_denied,
      redis_operations_total,
      redis_operations_failed,
      cache_hits,
      cache_misses,
      hydra_validations_total,
      hydra_validations_failed,
      request_duration_seconds,
      redis_operation_duration_seconds,
    })
  }

  pub fn record_token_check_success(&self) {
    self.token_checks_total.inc();
  }

  pub fn record_token_check_failure(&self) {
    self.token_checks_total.inc();
    self.token_checks_failed.inc();
  }

  pub fn record_token_revoked(&self) {
    self.tokens_revoked.inc();
  }

  pub fn record_auth_allowed(&self) {
    self.authorization_allowed.inc();
  }

  pub fn record_auth_denied(&self, reason: &str) {
    self.authorization_denied.with_label_values(&[reason]).inc();
  }

  pub fn record_redis_operation(&self, operation: &str) {
    self.redis_operations_total.with_label_values(&[operation]).inc();
  }

  pub fn record_redis_error(&self, operation: &str, error: &str) {
    self.redis_operations_failed.with_label_values(&[operation, error]).inc();
  }

  pub fn record_cache_hit(&self) {
    self.cache_hits.inc();
  }

  pub fn record_cache_miss(&self) {
    self.cache_misses.inc();
  }

  pub fn observe_redis_duration(&self, operation: &str, duration_secs: f64) {
    self.redis_operation_duration_seconds.with_label_values(&[operation]).observe(duration_secs);
  }

  pub fn record_hydra_validation(&self) {
    self.hydra_validations_total.inc();
  }

  pub fn record_hydra_validation_failure(&self) {
    self.hydra_validations_total.inc();
    self.hydra_validations_failed.inc();
  }

  pub fn observe_request_duration(&self, duration_secs: f64) {
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }
}
