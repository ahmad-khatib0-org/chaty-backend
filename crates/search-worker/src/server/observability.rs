use std::{
  convert::Infallible,
  io::{Error, ErrorKind},
  sync::Arc,
};

use chaty_config::Settings;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use http_body_util::Full;
use hyper::{
  body::{Bytes, Incoming},
  server::conn::http1::Builder,
  service::service_fn,
  Request, Response, StatusCode,
};
use hyper_util::rt::tokio::TokioIo;
use opentelemetry::{
  metrics::{Counter, Histogram, MeterProvider as _},
  KeyValue,
};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Registry, TextEncoder};
use tokio::{net::TcpListener, spawn};

/// OpenTelemetry + Prometheus metrics collector for the Search Worker service
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
  _provider: Arc<SdkMeterProvider>,
  // Message processing counters
  pub messages_processed_total: Counter<u64>,
  pub messages_failed_total: Counter<u64>,
  // Meilisearch operation metrics
  pub meili_indexing_duration_seconds: Histogram<f64>,
  pub meili_retries_total: Counter<u64>,
  pub meili_errors_total: Counter<u64>,
  // Kafka metrics
  pub kafka_messages_consumed_total: Counter<u64>,
  pub kafka_consume_errors_total: Counter<u64>,
  pub kafka_commit_errors_total: Counter<u64>,
}

impl Clone for MetricsCollector {
  fn clone(&self) -> Self {
    Self {
      config: self.config.clone(),
      registry: self.registry.clone(),
      _provider: self._provider.clone(),
      messages_processed_total: self.messages_processed_total.clone(),
      messages_failed_total: self.messages_failed_total.clone(),
      meili_indexing_duration_seconds: self.meili_indexing_duration_seconds.clone(),
      meili_retries_total: self.meili_retries_total.clone(),
      meili_errors_total: self.meili_errors_total.clone(),
      kafka_messages_consumed_total: self.kafka_messages_consumed_total.clone(),
      kafka_consume_errors_total: self.kafka_consume_errors_total.clone(),
      kafka_commit_errors_total: self.kafka_commit_errors_total.clone(),
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
      let path = "search-worker.server.observability".into();
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
    let meter = provider.meter("search-worker-service");
    let provider = Arc::new(provider);

    // --- Message Processing Metrics ---
    let messages_processed_total = meter
      .u64_counter("search_worker_messages_processed")
      .with_description("Total messages processed")
      .build();

    let messages_failed_total = meter
      .u64_counter("search_worker_messages_failed")
      .with_description("Total failed message processing")
      .build();

    // --- Meilisearch Metrics ---
    let meili_indexing_duration_seconds = meter
      .f64_histogram("search_worker_meili_indexing_duration_seconds")
      .with_description("Meilisearch indexing operation duration in seconds")
      .build();

    let meili_retries_total = meter
      .u64_counter("search_worker_meili_retries")
      .with_description("Total Meilisearch operation retries")
      .build();

    let meili_errors_total = meter
      .u64_counter("search_worker_meili_errors")
      .with_description("Total Meilisearch operation errors")
      .build();

    // --- Kafka Metrics ---
    let kafka_messages_consumed_total = meter
      .u64_counter("search_worker_kafka_messages_consumed")
      .with_description("Total Kafka messages consumed")
      .build();

    let kafka_consume_errors_total = meter
      .u64_counter("search_worker_kafka_consume_errors")
      .with_description("Total Kafka consumption errors")
      .build();

    let kafka_commit_errors_total = meter
      .u64_counter("search_worker_kafka_commit_errors")
      .with_description("Total Kafka offset commit errors")
      .build();

    Ok(MetricsCollector {
      registry: Arc::new(registry),
      config: args.config,
      _provider: provider,
      messages_processed_total,
      messages_failed_total,
      meili_indexing_duration_seconds,
      meili_retries_total,
      meili_errors_total,
      kafka_messages_consumed_total,
      kafka_consume_errors_total,
      kafka_commit_errors_total,
    })
  }
  /// Start HTTP server to expose metrics for Prometheus
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let url = self.config.hosts.search_metrics.clone();

    let listener = TcpListener::bind(&url).await?;
    let addr = listener.local_addr()?;
    tracing::info!("Search Worker Metrics server listening on {}", addr);

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
                  .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))
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

  pub fn record_message_processed(&self) {
    self.messages_processed_total.add(1, &[]);
  }

  pub fn record_message_failed(&self, index: &str) {
    self.messages_processed_total.add(1, &[]);
    self.messages_failed_total.add(1, &[KeyValue::new("index", index.to_string())]);
  }

  pub fn observe_meili_indexing_duration(&self, index: &str, duration_secs: f64) {
    self
      .meili_indexing_duration_seconds
      .record(duration_secs, &[KeyValue::new("index", index.to_string())]);
  }

  pub fn record_meili_retry(&self, index: &str) {
    self.meili_retries_total.add(1, &[KeyValue::new("index", index.to_string())]);
  }

  pub fn record_meili_error(&self, index: &str, error: &str) {
    self.meili_errors_total.add(
      1,
      &[
        KeyValue::new("index", index.to_string()),
        KeyValue::new("error", error.to_string()),
      ],
    );
  }

  pub fn record_kafka_message_consumed(&self, topic: &str) {
    self.kafka_messages_consumed_total.add(1, &[KeyValue::new("topic", topic.to_string())]);
  }

  pub fn record_kafka_consume_error(&self, topic: &str, error: &str) {
    self.kafka_consume_errors_total.add(
      1,
      &[
        KeyValue::new("topic", topic.to_string()),
        KeyValue::new("error", error.to_string()),
      ],
    );
  }

  pub fn record_kafka_commit_error(&self, topic: &str, partition: i32) {
    self.kafka_commit_errors_total.add(
      1,
      &[
        KeyValue::new("topic", topic.to_string()),
        KeyValue::new("partition", partition.to_string()),
      ],
    );
  }
}
