use std::io::{Error, ErrorKind};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use prometheus::{CounterVec, HistogramOpts, HistogramVec, IntCounter, Registry};

/// Prometheus metrics collector for API service
#[derive(Clone, Debug)]
pub struct MetricsCollector {
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

impl MetricsCollector {
  pub fn new(registry: &Registry) -> Result<Self, String> {
    let users_create_total = IntCounter::new("api_users_create_total", "Total user creations")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(users_create_total.clone())).map_err(|e| e.to_string())?;

    let users_create_failed =
      IntCounter::new("api_users_create_failed_total", "Total failed user creations")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(users_create_failed.clone())).map_err(|e| e.to_string())?;

    let users_get_total = IntCounter::new("api_users_get_total", "Total user retrieval requests")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(users_get_total.clone())).map_err(|e| e.to_string())?;

    let users_get_failed =
      IntCounter::new("api_users_get_failed_total", "Total failed user retrieval requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(users_get_failed.clone())).map_err(|e| e.to_string())?;

    let db_operations_total = CounterVec::new(
      prometheus::Opts::new("api_db_operations_total", "Total database operations"),
      &["operation"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(db_operations_total.clone())).map_err(|e| e.to_string())?;

    let db_operations_failed = CounterVec::new(
      prometheus::Opts::new("api_db_operations_failed_total", "Total failed database operations"),
      &["operation", "error"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(db_operations_failed.clone())).map_err(|e| e.to_string())?;

    let broker_messages_sent =
      IntCounter::new("api_broker_messages_sent_total", "Total messages sent to broker")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(broker_messages_sent.clone())).map_err(|e| e.to_string())?;

    let broker_messages_failed =
      IntCounter::new("api_broker_messages_failed_total", "Total failed broker messages")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(broker_messages_failed.clone())).map_err(|e| e.to_string())?;

    let request_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_request_duration_seconds", "Request duration in seconds"),
      &["endpoint"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(request_duration_seconds.clone())).map_err(|e| e.to_string())?;

    let db_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_db_operation_duration_seconds", "Database operation duration"),
      &["operation"],
    )
    .map_err(|e| e.to_string())?;
    registry
      .register(Box::new(db_operation_duration_seconds.clone()))
      .map_err(|e| e.to_string())?;

    let broker_operation_duration_seconds = HistogramVec::new(
      HistogramOpts::new("api_broker_operation_duration_seconds", "Broker operation duration"),
      &["operation"],
    )
    .map_err(|e| e.to_string())?;
    registry
      .register(Box::new(broker_operation_duration_seconds.clone()))
      .map_err(|e| e.to_string())?;

    Ok(MetricsCollector {
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

/// Initialize OpenTelemetry observability (Prometheus metrics and Jaeger tracing)
pub fn init_otel() -> Result<(Registry, MetricsCollector), BoxedErr> {
  let ie = |msg: &str, err: BoxedErr| {
    let path = "api.server.observability".into();
    InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
  };

  // Initialize Prometheus metrics registry
  let registry = Registry::new();

  // Create Prometheus exporter - metrics are pulled by OTel Collector at :8888
  // OTel Collector config routes these to Jaeger for visualization
  let _prometheus_exporter = opentelemetry_prometheus::exporter()
    .with_registry(registry.clone())
    .build()
    .map_err(|err| ie("failed to initialize prometheus exporter", Box::new(err)))?;

  // Create metrics collector
  let metrics = MetricsCollector::new(&registry).map_err(|e| {
    ie("failed to initialize metrics collector", Box::new(Error::new(ErrorKind::Other, e)))
  })?;

  Ok((registry, metrics))
}
