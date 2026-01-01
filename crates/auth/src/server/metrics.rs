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
use prometheus::{CounterVec, HistogramOpts, HistogramVec, IntCounter, Registry, TextEncoder};
use tokio::{net::TcpListener, spawn};

/// Prometheus metrics collector for auth service
#[derive(Clone, Debug)]
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
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

pub struct MetricsCollectorArgs {
  pub config: Arc<Settings>,
}

impl MetricsCollector {
  pub fn new(args: MetricsCollectorArgs) -> Result<Self, BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| {
      let path = "auth.controller.metrics".into();
      InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
    };

    // Initialize Prometheus metrics registry
    let registry = Registry::new();

    // Create Prometheus exporter
    let _prometheus_exporter = opentelemetry_prometheus::exporter()
      .with_registry(registry.clone())
      .build()
      .map_err(|err| ie("failed to initialize prometheus exporter", Box::new(err)))?;

    // --- Token Metrics ---
    let token_checks_total = IntCounter::new("auth_token_checks_total", "Total token checks")
      .map_err(|err| ie("failed to create token_checks_total", Box::new(err)))?;
    registry
      .register(Box::new(token_checks_total.clone()))
      .map_err(|err| ie("failed to register token_checks_total", Box::new(err)))?;

    let token_checks_failed =
      IntCounter::new("auth_token_checks_failed_total", "Total failed token checks")
        .map_err(|err| ie("failed to create token_checks_failed", Box::new(err)))?;
    registry
      .register(Box::new(token_checks_failed.clone()))
      .map_err(|err| ie("failed to register token_checks_failed", Box::new(err)))?;

    let tokens_revoked = IntCounter::new("auth_tokens_revoked_total", "Total tokens revoked")
      .map_err(|err| ie("failed to create tokens_revoked", Box::new(err)))?;
    registry
      .register(Box::new(tokens_revoked.clone()))
      .map_err(|err| ie("failed to register tokens_revoked", Box::new(err)))?;

    // --- Authorization Metrics ---
    let authorization_allowed =
      IntCounter::new("auth_authorization_allowed_total", "Total successful authorizations")
        .map_err(|err| ie("failed to create authorization_allowed", Box::new(err)))?;
    registry
      .register(Box::new(authorization_allowed.clone()))
      .map_err(|err| ie("failed to register authorization_allowed", Box::new(err)))?;

    let authorization_denied = CounterVec::new(
      prometheus::Opts::new("auth_authorization_denied_total", "Total denied authorizations"),
      &["reason"],
    )
    .map_err(|err| ie("failed to create authorization_denied", Box::new(err)))?;
    registry
      .register(Box::new(authorization_denied.clone()))
      .map_err(|err| ie("failed to register authorization_denied", Box::new(err)))?;

    // --- Redis Metrics ---
    let redis_operations_total = CounterVec::new(
      prometheus::Opts::new("auth_redis_operations_total", "Total Redis operations"),
      &["operation"],
    )
    .map_err(|err| ie("failed to create redis_operations_total", Box::new(err)))?;
    registry
      .register(Box::new(redis_operations_total.clone()))
      .map_err(|err| ie("failed to register redis_operations_total", Box::new(err)))?;

    let redis_operations_failed = CounterVec::new(
      prometheus::Opts::new("auth_redis_operations_failed_total", "Total failed Redis operations"),
      &["operation", "error"],
    )
    .map_err(|err| ie("failed to create redis_operations_failed", Box::new(err)))?;
    registry
      .register(Box::new(redis_operations_failed.clone()))
      .map_err(|err| ie("failed to register redis_operations_failed", Box::new(err)))?;

    // --- Cache Metrics ---
    let cache_hits = IntCounter::new("auth_cache_hits_total", "Total cache hits")
      .map_err(|err| ie("failed to create cache_hits", Box::new(err)))?;
    registry
      .register(Box::new(cache_hits.clone()))
      .map_err(|err| ie("failed to register cache_hits", Box::new(err)))?;

    let cache_misses = IntCounter::new("auth_cache_misses_total", "Total cache misses")
      .map_err(|err| ie("failed to create cache_misses", Box::new(err)))?;
    registry
      .register(Box::new(cache_misses.clone()))
      .map_err(|err| ie("failed to register cache_misses", Box::new(err)))?;

    // --- Hydra Metrics ---
    let hydra_validations_total =
      IntCounter::new("auth_hydra_validations_total", "Total Hydra validations")
        .map_err(|err| ie("failed to create hydra_validations_total", Box::new(err)))?;
    registry
      .register(Box::new(hydra_validations_total.clone()))
      .map_err(|err| ie("failed to register hydra_validations_total", Box::new(err)))?;

    let hydra_validations_failed =
      IntCounter::new("auth_hydra_validations_failed_total", "Total failed Hydra validations")
        .map_err(|err| ie("failed to create hydra_validations_failed", Box::new(err)))?;
    registry
      .register(Box::new(hydra_validations_failed.clone()))
      .map_err(|err| ie("failed to register hydra_validations_failed", Box::new(err)))?;

    // --- Duration Histograms ---
    let request_duration_seconds = HistogramVec::new(
      HistogramOpts::new("auth_request_duration_seconds", "Request duration in seconds"),
      &[],
    )
    .map_err(|err| ie("failed to create request_duration_seconds", Box::new(err)))?;
    registry
      .register(Box::new(request_duration_seconds.clone()))
      .map_err(|err| ie("failed to register request_duration_seconds", Box::new(err)))?;

    let redis_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("auth_redis_operation_duration_seconds", "Redis operation duration"),
      &["operation"],
    )
    .map_err(|err| ie("failed to create redis_operation_duration", Box::new(err)))?;
    registry
      .register(Box::new(redis_operation_duration_seconds.clone()))
      .map_err(|err| ie("failed to register redis_operation_duration_seconds", Box::new(err)))?;

    Ok(MetricsCollector {
      registry: Arc::new(registry),
      config: args.config,
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

                Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
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
