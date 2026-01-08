use std::{io::ErrorKind, sync::Arc};

use chaty_config::{config, Settings};
use chaty_database::{DatabaseInfoSql, DatabaseSql};
use chaty_result::errors::{BoxedErr, ErrorType, SimpleError};
use tokio::spawn;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

use crate::{
  controller::{SearchWorkerController, SearchWorkerControllerArgs},
  server::observability::{MetricsCollector, MetricsCollectorArgs},
};

pub mod observability;

pub struct SearchWorkerServer {
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) metrics: Arc<MetricsCollector>,
}

impl SearchWorkerServer {
  pub async fn new() -> Result<SearchWorkerServer, BoxedErr> {
    let se = |err: BoxedErr, typ: ErrorType, msg: &str| {
      return SimpleError { err, err_type: typ, message: msg.to_string() };
    };

    SearchWorkerServer::setup_logging();
    let config = config().await;

    // Initialize observability
    let metrics = MetricsCollector::new(MetricsCollectorArgs { config: Arc::new(config.clone()) })?;

    let sql_db = DatabaseInfoSql::Postgres { dsn: config.database.postgres.clone() }
      .connect()
      .await
      .map_err(|err| {
        se(Box::new(std::io::Error::new(ErrorKind::NotConnected, err)), ErrorType::Connection, "")
      })?;

    let server = SearchWorkerServer {
      sql_db: Arc::new(sql_db),
      config: Arc::new(config),
      metrics: Arc::new(metrics),
    };

    Ok(server)
  }

  /// call the run of the grpc server
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let ctr_args = SearchWorkerControllerArgs {
      sql_db: self.sql_db.clone(),
      config: self.config.clone(),
      metrics: self.metrics.clone(),
    };

    let metrics_clone = self.metrics.clone();
    spawn(async move {
      if let Err(e) = metrics_clone.run().await {
        error!("Metrics server failed: {:?}", e);
      }
    });

    let controller = SearchWorkerController::new(ctr_args);
    controller.run().await?; // this will block

    Ok(())
  }

  fn setup_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber =
      tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
  }
}
