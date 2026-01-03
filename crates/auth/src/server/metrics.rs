use std::{convert::Infallible, sync::Arc};

use chaty_config::Settings;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use http_body_util::Full;
use hyper::{
  body::{Bytes, Incoming},
  server::conn::http1::Builder,
  service::service_fn,
  Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use opentelemetry::{
  metrics::{Counter, Histogram, MeterProvider as _},
  KeyValue,
};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Registry, TextEncoder};
use tokio::{net::TcpListener, spawn};

/// OpenTelemetry + Prometheus metrics collector for auth service
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
  _provider: Arc<SdkMeterProvider>,
  // Counters
  pub token_checks_total: Counter<u64>,
  pub token_checks_failed: Counter<u64>,
  pub tokens_revoked: Counter<u64>,
  pub authorization_allowed: Counter<u64>,
  pub authorization_denied: Counter<u64>,
  pub redis_operations_total: Counter<u64>,
  pub redis_operations_failed: Counter<u64>,
  pub cache_hits: Counter<u64>,
  pub cache_misses: Counter<u64>,
  pub hydra_validations_total: Counter<u64>,
  pub hydra_validations_failed: Counter<u64>,
  // Histograms
  pub request_duration_seconds: Histogram<f64>,
  pub redis_operation_duration_seconds: Histogram<f64>,
}

impl Clone for MetricsCollector {
  fn clone(&self) -> Self {
    Self {
      config: self.config.clone(),
      registry: self.registry.clone(),
      _provider: self._provider.clone(),
      token_checks_total: self.token_checks_total.clone(),
      token_checks_failed: self.token_checks_failed.clone(),
      tokens_revoked: self.tokens_revoked.clone(),
      authorization_allowed: self.authorization_allowed.clone(),
      authorization_denied: self.authorization_denied.clone(),
      redis_operations_total: self.redis_operations_total.clone(),
      redis_operations_failed: self.redis_operations_failed.clone(),
      cache_hits: self.cache_hits.clone(),
      cache_misses: self.cache_misses.clone(),
      hydra_validations_total: self.hydra_validations_total.clone(),
      hydra_validations_failed: self.hydra_validations_failed.clone(),
      request_duration_seconds: self.request_duration_seconds.clone(),
      redis_operation_duration_seconds: self.redis_operation_duration_seconds.clone(),
    }
  }
}

impl std::fmt::Debug for MetricsCollector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MetricsCollector").finish()
  }
}

pub struct MetricsCollectorArgs {
  pub config: Arc<Settings>,
}

impl MetricsCollector {
  pub fn new(args: MetricsCollectorArgs) -> Result<Self, BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| {
      let path = "auth.server.metrics".into();
      InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
    };

    // Initialize Prometheus registry
    let registry = Registry::new();

    // Create OpenTelemetry Prometheus exporter
    let exporter = opentelemetry_prometheus::exporter()
      .with_registry(registry.clone())
      .build()
      .map_err(|err| ie("failed to initialize prometheus exporter", Box::new(err)))?;

    // Create meter provider with Prometheus exporter
    let provider = SdkMeterProvider::builder().with_reader(exporter).build();
    let meter = provider.meter("auth-service");
    let provider = Arc::new(provider);

    // --- Token Metrics ---
    let token_checks_total =
      meter.u64_counter("auth_token_checks").with_description("Total token checks").build();

    let token_checks_failed = meter
      .u64_counter("auth_token_checks_failed")
      .with_description("Total failed token checks")
      .build();

    let tokens_revoked =
      meter.u64_counter("auth_tokens_revoked").with_description("Total tokens revoked").build();

    // --- Authorization Metrics ---
    let authorization_allowed = meter
      .u64_counter("auth_authorization_allowed")
      .with_description("Total successful authorizations")
      .build();

    let authorization_denied = meter
      .u64_counter("auth_authorization_denied")
      .with_description("Total denied authorizations")
      .build();

    // --- Redis Metrics ---
    let redis_operations_total =
      meter.u64_counter("auth_redis_operations").with_description("Total Redis operations").build();

    let redis_operations_failed = meter
      .u64_counter("auth_redis_operations_failed")
      .with_description("Total failed Redis operations")
      .build();

    // --- Cache Metrics ---
    let cache_hits =
      meter.u64_counter("auth_cache_hits").with_description("Total cache hits").build();

    let cache_misses =
      meter.u64_counter("auth_cache_misses").with_description("Total cache misses").build();

    // --- Hydra Metrics ---
    let hydra_validations_total = meter
      .u64_counter("auth_hydra_validations")
      .with_description("Total Hydra validations")
      .build();

    let hydra_validations_failed = meter
      .u64_counter("auth_hydra_validations_failed")
      .with_description("Total failed Hydra validations")
      .build();

    // --- Duration Histograms ---
    let request_duration_seconds = meter
      .f64_histogram("auth_request_duration_seconds")
      .with_description("Request duration in seconds")
      .build();

    let redis_operation_duration_seconds = meter
      .f64_histogram("auth_redis_operation_duration_seconds")
      .with_description("Redis operation duration in seconds")
      .build();

    Ok(MetricsCollector {
      registry: Arc::new(registry),
      config: args.config,
      _provider: provider,
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
  /// Start HTTP server to expose metrics for Prometheus
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let url = self.config.hosts.auth_metrics.clone();

    let listener = TcpListener::bind(&url).await?;
    let addr = listener.local_addr()?;
    tracing::info!("AUTH Metrics server listening on {}", addr);

    loop {
      let (socket, _) = listener.accept().await?;
      let io = TokioIo::new(socket);

      let connection_registry = self.registry.clone();

      spawn(async move {
        let svc = service_fn(move |req: Request<Incoming>| {
          let request_registry = connection_registry.clone();

          async move {
            let path = req.uri().path();
            match path {
              "/metrics" => {
                let encoder = TextEncoder::new();
                let body = encoder
                  .encode_to_string(&request_registry.gather())
                  .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))
                  .unwrap_or_default();

                Ok::<_, Infallible>(
                  Response::builder()
                    .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap(),
                )
              }
              "/health" => Ok(Response::new(Full::new(Bytes::from_static(b"OK")))),
              _ => Ok(
                Response::builder()
                  .status(StatusCode::NOT_FOUND)
                  .body(Full::new(Bytes::from_static(b"Not Found")))
                  .unwrap(),
              ),
            }
          }
        });

        if let Err(err) = Builder::new().serve_connection(io, svc).await {
          tracing::error!("Error serving metrics: {}", err);
        }
      });
    }
  }

  pub fn record_token_check_success(&self) {
    self.token_checks_total.add(1, &[]);
  }

  pub fn record_token_check_failure(&self) {
    self.token_checks_total.add(1, &[]);
    self.token_checks_failed.add(1, &[]);
  }

  pub fn record_token_revoked(&self) {
    self.tokens_revoked.add(1, &[]);
  }

  pub fn record_auth_allowed(&self) {
    self.authorization_allowed.add(1, &[]);
  }

  pub fn record_auth_denied(&self, reason: &str) {
    self.authorization_denied.add(1, &[KeyValue::new("reason", reason.to_string())]);
  }

  pub fn record_redis_operation(&self, operation: &str) {
    self.redis_operations_total.add(1, &[KeyValue::new("operation", operation.to_string())]);
  }

  pub fn record_redis_error(&self, operation: &str, error: &str) {
    self.redis_operations_failed.add(
      1,
      &[
        KeyValue::new("operation", operation.to_string()),
        KeyValue::new("error", error.to_string()),
      ],
    );
  }

  pub fn record_cache_hit(&self) {
    self.cache_hits.add(1, &[]);
  }

  pub fn record_cache_miss(&self) {
    self.cache_misses.add(1, &[]);
  }

  pub fn observe_redis_duration(&self, operation: &str, duration_secs: f64) {
    self
      .redis_operation_duration_seconds
      .record(duration_secs, &[KeyValue::new("operation", operation.to_string())]);
  }

  pub fn record_hydra_validation(&self) {
    self.hydra_validations_total.add(1, &[]);
  }

  pub fn record_hydra_validation_failure(&self) {
    self.hydra_validations_total.add(1, &[]);
    self.hydra_validations_failed.add(1, &[]);
  }

  pub fn observe_request_duration(&self, duration_secs: f64) {
    self.request_duration_seconds.record(duration_secs, &[]);
  }
}
