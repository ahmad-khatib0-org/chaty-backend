use std::env;

use prometheus::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::controller::metrics::MetricsCollector;

pub fn init_otel(_service_name: &str) -> Result<(Registry, MetricsCollector), (String, String)> {
  let _otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
    .unwrap_or_else(|_| "http://otel-collector:4317".to_string());

  // Initialize Prometheus metrics registry that will be scraped by OTel Collector
  let registry = Registry::new();

  // Create Prometheus exporter - metrics are pulled by OTel Collector at :8888
  // OTel Collector config routes these to Jaeger for visualization
  let _prometheus_exporter =
    opentelemetry_prometheus::exporter().with_registry(registry.clone()).build().map_err(|e| {
      let err_str = format!("prometheus setup failed: {}", e);
      (err_str.clone(), err_str)
    })?;

  // Create metrics collector
  let metrics = MetricsCollector::new(&registry).map_err(|e| (e.clone(), e))?;

  // Set up environment filter for logs
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

  // Initialize tracing subscriber for structured logging
  let subscriber =
    tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer());

  let _ = tracing::subscriber::set_default(subscriber);

  Ok((registry, metrics))
}
