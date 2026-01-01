use std::io::{Error, ErrorKind};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use prometheus::Registry;

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

  Ok((registry, metrics))
}
