use std::io::{Error, ErrorKind};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use prometheus::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::controller::metrics::MetricsCollector;

pub fn init_otel() -> Result<(Registry, MetricsCollector), BoxedErr> {
  let ie = |msg: &str, err: BoxedErr| {
    let path = "auth.controller.run".into();
    return InternalError {
      err_type: ErrorType::InternalError,
      temp: false,
      err,
      msg: msg.into(),
      path,
    };
  };

  // Initialize Prometheus metrics registry that will be scraped by OTel Collector
  let registry = Registry::new();

  // Create Prometheus exporter - metrics are pulled by OTel Collector at :8888
  // OTel Collector config routes these to Jaeger for visualization
  let _prometheus_exporter = opentelemetry_prometheus::exporter()
    .with_registry(registry.clone())
    .build()
    .map_err(|err| ie("failed to init metrics exporter", Box::new(err)))?;

  // Create metrics collector
  let metrics = MetricsCollector::new(&registry).map_err(|e| {
    ie("failed to init metrics collector", Box::new(Error::new(ErrorKind::Other, e)))
  })?;

  // Set up environment filter for logs
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

  // Initialize tracing subscriber for structured logging
  let subscriber =
    tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer());

  let _ = tracing::subscriber::set_default(subscriber);

  Ok((registry, metrics))
}
