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
use opentelemetry::{
  metrics::{Counter, Histogram, MeterProvider as _},
  KeyValue,
};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Registry, TextEncoder};
use tokio::net::TcpListener;
use tokio::spawn;

/// OpenTelemetry + Prometheus metrics collector for API service
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
  _provider: Arc<SdkMeterProvider>,
  // Counters
  pub users_create_total: Counter<u64>,
  pub users_create_failed: Counter<u64>,
  pub users_login_total: Counter<u64>,
  pub users_login_failed: Counter<u64>,
  pub users_email_confirmation_total: Counter<u64>,
  pub users_email_confirmation_failed: Counter<u64>,
  pub users_forgot_password_total: Counter<u64>,
  pub users_forgot_password_failed: Counter<u64>,
  pub users_reset_password_total: Counter<u64>,
  pub users_reset_password_failed: Counter<u64>,
  pub users_get_total: Counter<u64>,
  pub users_get_failed: Counter<u64>,
  pub groups_create_total: Counter<u64>,
  pub groups_create_failed: Counter<u64>,
  pub search_usernames_total: Counter<u64>,
  pub search_usernames_failed: Counter<u64>,
  pub db_operations_total: Counter<u64>,
  pub db_operations_failed: Counter<u64>,
  pub broker_messages_sent: Counter<u64>,
  pub broker_messages_failed: Counter<u64>,
  // Histograms
  pub request_duration_seconds: Histogram<f64>,
  pub db_operation_duration_seconds: Histogram<f64>,
  pub broker_operation_duration_seconds: Histogram<f64>,
}

impl Clone for MetricsCollector {
  fn clone(&self) -> Self {
    Self {
      config: self.config.clone(),
      registry: self.registry.clone(),
      _provider: self._provider.clone(),
      users_create_total: self.users_create_total.clone(),
      users_create_failed: self.users_create_failed.clone(),
      users_login_total: self.users_login_total.clone(),
      users_login_failed: self.users_login_failed.clone(),
      users_email_confirmation_total: self.users_email_confirmation_total.clone(),
      users_email_confirmation_failed: self.users_email_confirmation_failed.clone(),
      users_forgot_password_total: self.users_forgot_password_total.clone(),
      users_forgot_password_failed: self.users_forgot_password_failed.clone(),
      users_reset_password_total: self.users_reset_password_total.clone(),
      users_reset_password_failed: self.users_reset_password_failed.clone(),
      users_get_total: self.users_get_total.clone(),
      users_get_failed: self.users_get_failed.clone(),
      groups_create_total: self.groups_create_total.clone(),
      groups_create_failed: self.groups_create_failed.clone(),
      search_usernames_total: self.search_usernames_total.clone(),
      search_usernames_failed: self.search_usernames_failed.clone(),
      db_operations_total: self.db_operations_total.clone(),
      db_operations_failed: self.db_operations_failed.clone(),
      broker_messages_sent: self.broker_messages_sent.clone(),
      broker_messages_failed: self.broker_messages_failed.clone(),
      request_duration_seconds: self.request_duration_seconds.clone(),
      db_operation_duration_seconds: self.db_operation_duration_seconds.clone(),
      broker_operation_duration_seconds: self.broker_operation_duration_seconds.clone(),
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
      let path = "api.server.observability".into();
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
    let meter = provider.meter("api-service");
    let provider = Arc::new(provider);

    // --- Users Metrics ---
    let users_create_total =
      meter.u64_counter("api_users_create").with_description("Total user creations").build();

    let users_create_failed = meter
      .u64_counter("api_users_create_failed")
      .with_description("Total failed user creations")
      .build();

    let users_login_total =
      meter.u64_counter("api_users_login").with_description("Total user logins").build();

    let users_login_failed = meter
      .u64_counter("api_users_login_failed")
      .with_description("Total failed user logins")
      .build();

    let users_email_confirmation_total = meter
      .u64_counter("api_users_email_confirmation")
      .with_description("Total email confirmation requests")
      .build();

    let users_email_confirmation_failed = meter
      .u64_counter("api_users_email_confirmation_failed")
      .with_description("Total failed email confirmation requests")
      .build();

    let users_forgot_password_total = meter
      .u64_counter("api_users_forgot_password")
      .with_description("Total forgot password requests")
      .build();

    let users_forgot_password_failed = meter
      .u64_counter("api_users_forgot_password_failed")
      .with_description("Total failed forgot password requests")
      .build();

    let users_reset_password_total = meter
      .u64_counter("api_users_reset_password")
      .with_description("Total password reset requests")
      .build();

    let users_reset_password_failed = meter
      .u64_counter("api_users_reset_password_failed")
      .with_description("Total failed password reset requests")
      .build();

    let users_get_total =
      meter.u64_counter("api_users_get").with_description("Total user retrieval requests").build();

    let users_get_failed = meter
      .u64_counter("api_users_get_failed")
      .with_description("Total failed user retrieval requests")
      .build();

    // --- Groups Metrics ---
    let groups_create_total =
      meter.u64_counter("api_groups_create").with_description("Total group creations").build();

    let groups_create_failed = meter
      .u64_counter("api_groups_create_failed")
      .with_description("Total failed group creations")
      .build();

    // --- Search Metrics ---
    let search_usernames_total = meter
      .u64_counter("api_search_usernames")
      .with_description("Total username search requests")
      .build();

    let search_usernames_failed = meter
      .u64_counter("api_search_usernames_failed")
      .with_description("Total failed username search requests")
      .build();

    // --- Database Metrics ---
    let db_operations_total =
      meter.u64_counter("api_db_operations").with_description("Total database operations").build();

    let db_operations_failed = meter
      .u64_counter("api_db_operations_failed")
      .with_description("Total failed database operations")
      .build();

    // --- Broker Metrics ---
    let broker_messages_sent = meter
      .u64_counter("api_broker_messages_sent")
      .with_description("Total messages sent to broker")
      .build();

    let broker_messages_failed = meter
      .u64_counter("api_broker_messages_failed")
      .with_description("Total failed broker messages")
      .build();

    // --- Duration Histograms ---
    let request_duration_seconds = meter
      .f64_histogram("api_request_duration_seconds")
      .with_description("Request duration in seconds")
      .build();

    let db_operation_duration_seconds = meter
      .f64_histogram("api_db_operation_duration_seconds")
      .with_description("Database operation duration in seconds")
      .build();

    let broker_operation_duration_seconds = meter
      .f64_histogram("api_broker_operation_duration_seconds")
      .with_description("Broker operation duration in seconds")
      .build();

    Ok(MetricsCollector {
      registry: Arc::new(registry),
      config: args.config,
      _provider: provider,
      users_create_total,
      users_create_failed,
      users_login_total,
      users_login_failed,
      users_email_confirmation_total,
      users_email_confirmation_failed,
      users_forgot_password_total,
      users_forgot_password_failed,
      users_reset_password_total,
      users_reset_password_failed,
      users_get_total,
      users_get_failed,
      groups_create_total,
      groups_create_failed,
      search_usernames_total,
      search_usernames_failed,
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
    self.users_create_total.add(1, &[]);
  }

  pub fn record_users_create_failure(&self) {
    self.users_create_total.add(1, &[]);
    self.users_create_failed.add(1, &[]);
  }

  pub fn record_users_get_success(&self) {
    self.users_get_total.add(1, &[]);
  }

  pub fn record_users_get_failure(&self) {
    self.users_get_total.add(1, &[]);
    self.users_get_failed.add(1, &[]);
  }

  pub fn record_users_login_success(&self) {
    self.users_login_total.add(1, &[]);
  }

  pub fn record_users_login_failure(&self) {
    self.users_login_total.add(1, &[]);
    self.users_login_failed.add(1, &[]);
  }

  pub fn record_users_email_confirmation_success(&self) {
    self.users_email_confirmation_total.add(1, &[]);
  }

  pub fn record_users_email_confirmation_failure(&self) {
    self.users_email_confirmation_total.add(1, &[]);
    self.users_email_confirmation_failed.add(1, &[]);
  }

  pub fn record_users_forgot_password_success(&self) {
    self.users_forgot_password_total.add(1, &[]);
  }

  pub fn record_users_forgot_password_failure(&self) {
    self.users_forgot_password_total.add(1, &[]);
    self.users_forgot_password_failed.add(1, &[]);
  }

  pub fn record_users_reset_password_success(&self) {
    self.users_reset_password_total.add(1, &[]);
  }

  pub fn record_users_reset_password_failure(&self) {
    self.users_reset_password_total.add(1, &[]);
    self.users_reset_password_failed.add(1, &[]);
  }

  pub fn record_groups_create_success(&self) {
    self.groups_create_total.add(1, &[]);
  }

  pub fn record_groups_create_failure(&self) {
    self.groups_create_total.add(1, &[]);
    self.groups_create_failed.add(1, &[]);
  }

  pub fn record_search_usernames_success(&self) {
    self.search_usernames_total.add(1, &[]);
  }

  pub fn record_search_usernames_failure(&self) {
    self.search_usernames_total.add(1, &[]);
    self.search_usernames_failed.add(1, &[]);
  }

  pub fn record_db_operation(&self, operation: &str) {
    self.db_operations_total.add(1, &[KeyValue::new("operation", operation.to_string())]);
  }

  pub fn record_db_error(&self, operation: &str, error: &str) {
    self.db_operations_failed.add(
      1,
      &[
        KeyValue::new("operation", operation.to_string()),
        KeyValue::new("error", error.to_string()),
      ],
    );
  }

  pub fn record_broker_message_sent(&self) {
    self.broker_messages_sent.add(1, &[]);
  }

  pub fn record_broker_message_failed(&self) {
    self.broker_messages_failed.add(1, &[]);
  }

  pub fn observe_request_duration(&self, endpoint: &str, duration_secs: f64) {
    self
      .request_duration_seconds
      .record(duration_secs, &[KeyValue::new("endpoint", endpoint.to_string())]);
  }

  pub fn observe_db_operation_duration(&self, operation: &str, duration_secs: f64) {
    self
      .db_operation_duration_seconds
      .record(duration_secs, &[KeyValue::new("operation", operation.to_string())]);
  }

  pub fn observe_broker_operation_duration(&self, operation: &str, duration_secs: f64) {
    self
      .broker_operation_duration_seconds
      .record(duration_secs, &[KeyValue::new("operation", operation.to_string())]);
  }
}
