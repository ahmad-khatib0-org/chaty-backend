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
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Registry, TextEncoder};
use tokio::{net::TcpListener, spawn};

/// OpenTelemetry + Prometheus metrics collector for the Search Worker service
pub struct MetricsCollector {
  config: Arc<Settings>,
  registry: Arc<Registry>,
  _provider: Arc<SdkMeterProvider>,
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
    let meter = provider.meter("api-service");
    let provider = Arc::new(provider);

    Ok(MetricsCollector { registry: Arc::new(registry), config: args.config, _provider: provider })
  }
  /// Start HTTP server to expose metrics for Prometheus
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let url = self.config.hosts.search_metrics.clone();

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
}
