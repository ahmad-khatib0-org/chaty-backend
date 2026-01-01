use std::convert::Infallible;
use std::sync::Arc;

use chaty_config::Settings;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{
  body::Bytes, server::conn::http1::Builder, service::service_fn, Request, Response, StatusCode,
};
use hyper_util::rt::tokio::TokioIo;
use prometheus::{CounterVec, HistogramOpts, HistogramVec, IntCounter, Registry, TextEncoder};
use tokio::net::TcpListener;
use tokio::spawn;

/// Prometheus metrics collector for API service
#[derive(Clone, Debug)]
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
  pub users_create_total: IntCounter,
  pub users_create_failed: IntCounter,
  pub users_get_total: IntCounter,
  pub users_get_failed: IntCounter,
  pub db_operations_total: CounterVec,
  pub db_operations_failed: CounterVec,
  pub broker_messages_sent: IntCounter,
  pub broker_messages_failed: IntCounter,
  pub request_duration_seconds: HistogramVec,
  pub db_operation_duration_seconds: HistogramVec,
  pub broker_operation_duration_seconds: HistogramVec,
}

pub struct MetricsCollectorArgs {
  pub config: Arc<Settings>,
}

impl MetricsCollector {
  pub fn new(args: MetricsCollectorArgs) -> Result<Self, BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| {
      let path = "api.server.observability".into();
      InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
    };

    // Initialize Prometheus metrics registry
    let registry = Registry::new();

    // // Create Prometheus exporter
    // let _prometheus_exporter = opentelemetry_prometheus::exporter()
    //   .with_registry(registry.clone())
    //   .build()
    //   .map_err(|err| ie("failed to initialize prometheus exporter", Box::new(err)))?;

    // --- Users Metrics ---
    let users_create_total = IntCounter::new("api_users_create_total", "Total user creations")
      .map_err(|err| ie("failed to create users_create_total counter", Box::new(err)))?;
    registry
      .register(Box::new(users_create_total.clone()))
      .map_err(|err| ie("failed to register users_create_total", Box::new(err)))?;

    let users_create_failed =
      IntCounter::new("api_users_create_failed_total", "Total failed user creations")
        .map_err(|err| ie("failed to create users_create_failed counter", Box::new(err)))?;
    registry
      .register(Box::new(users_create_failed.clone()))
      .map_err(|err| ie("failed to register users_create_failed", Box::new(err)))?;

    let users_get_total =
      IntCounter::new("api_users_get_total", "Total user retrieval requests")
        .map_err(|err| ie("failed to create users_get_total counter", Box::new(err)))?;
    registry
      .register(Box::new(users_get_total.clone()))
      .map_err(|err| ie("failed to register users_get_total", Box::new(err)))?;

    let users_get_failed =
      IntCounter::new("api_users_get_failed_total", "Total failed user retrieval requests")
        .map_err(|err| ie("failed to create users_get_failed counter", Box::new(err)))?;
    registry
      .register(Box::new(users_get_failed.clone()))
      .map_err(|err| ie("failed to register users_get_failed", Box::new(err)))?;

    // --- Database Metrics ---
    let db_operations_total = CounterVec::new(
      prometheus::Opts::new("api_db_operations_total", "Total database operations"),
      &["operation"],
    )
    .map_err(|err| ie("failed to create db_operations_total counter", Box::new(err)))?;
    registry
      .register(Box::new(db_operations_total.clone()))
      .map_err(|err| ie("failed to register db_operations_total", Box::new(err)))?;

    let db_operations_failed = CounterVec::new(
      prometheus::Opts::new("api_db_operations_failed_total", "Total failed database operations"),
      &["operation", "error"],
    )
    .map_err(|err| ie("failed to create db_operations_failed counter", Box::new(err)))?;
    registry
      .register(Box::new(db_operations_failed.clone()))
      .map_err(|err| ie("failed to register db_operations_failed", Box::new(err)))?;

    // --- Broker Metrics ---
    let broker_messages_sent =
      IntCounter::new("api_broker_messages_sent_total", "Total messages sent to broker")
        .map_err(|err| ie("failed to create broker_messages_sent counter", Box::new(err)))?;
    registry
      .register(Box::new(broker_messages_sent.clone()))
      .map_err(|err| ie("failed to register broker_messages_sent", Box::new(err)))?;

    let broker_messages_failed =
      IntCounter::new("api_broker_messages_failed_total", "Total failed broker messages")
        .map_err(|err| ie("failed to create broker_messages_failed counter", Box::new(err)))?;
    registry
      .register(Box::new(broker_messages_failed.clone()))
      .map_err(|err| ie("failed to register broker_messages_failed", Box::new(err)))?;

    // --- Duration Histograms ---
    let request_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_request_duration_seconds", "Request duration in seconds"),
      &["endpoint"],
    )
    .map_err(|err| ie("failed to create request_duration histogram", Box::new(err)))?;
    registry
      .register(Box::new(request_duration_seconds.clone()))
      .map_err(|err| ie("failed to register request_duration_seconds", Box::new(err)))?;

    let db_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_db_operation_duration_seconds", "Database operation duration"),
      &["operation"],
    )
    .map_err(|err| ie("failed to create db_duration histogram", Box::new(err)))?;
    registry
      .register(Box::new(db_operation_duration_seconds.clone()))
      .map_err(|err| ie("failed to register db_operation_duration_seconds", Box::new(err)))?;

    let broker_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_broker_operation_duration_seconds", "Broker operation duration"),
      &["operation"],
    )
    .map_err(|err| ie("failed to create broker_duration histogram", Box::new(err)))?;
    registry
      .register(Box::new(broker_operation_duration_seconds.clone()))
      .map_err(|err| ie("failed to register broker_operation_duration_seconds", Box::new(err)))?;

    Ok(MetricsCollector {
      registry: Arc::new(registry),
      config: args.config,
      users_create_total,
      users_create_failed,
      users_get_total,
      users_get_failed,
      db_operations_total,
      db_operations_failed,
      broker_messages_sent,
      broker_messages_failed,
      request_duration_seconds,
      db_operation_duration_seconds,
      broker_operation_duration_seconds,
    })
  }

  /// Start HTTP server to expose metrics for Prometheus
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let url = self.config.hosts.api_metrics.clone();

    let listener = TcpListener::bind(&url).await?;
    let addr = listener.local_addr()?;
    tracing::info!("API Metrics server listening on {}", addr);

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

  pub fn record_users_create_success(&self) {
    self.users_create_total.inc();
  }

  pub fn record_users_create_failure(&self) {
    self.users_create_total.inc();
    self.users_create_failed.inc();
  }

  pub fn record_users_get_success(&self) {
    self.users_get_total.inc();
  }

  pub fn record_users_get_failure(&self) {
    self.users_get_total.inc();
    self.users_get_failed.inc();
  }

  pub fn record_db_operation(&self, operation: &str) {
    self.db_operations_total.with_label_values(&[operation]).inc();
  }

  pub fn record_db_error(&self, operation: &str, error: &str) {
    self.db_operations_failed.with_label_values(&[operation, error]).inc();
  }

  pub fn record_broker_message_sent(&self) {
    self.broker_messages_sent.inc();
  }

  pub fn record_broker_message_failed(&self) {
    self.broker_messages_failed.inc();
  }

  pub fn observe_request_duration(&self, endpoint: &str, duration_secs: f64) {
    self.request_duration_seconds.with_label_values(&[endpoint]).observe(duration_secs);
  }

  pub fn observe_db_operation_duration(&self, operation: &str, duration_secs: f64) {
    self.db_operation_duration_seconds.with_label_values(&[operation]).observe(duration_secs);
  }

  pub fn observe_broker_operation_duration(&self, operation: &str, duration_secs: f64) {
    self.broker_operation_duration_seconds.with_label_values(&[operation]).observe(duration_secs);
  }
}
