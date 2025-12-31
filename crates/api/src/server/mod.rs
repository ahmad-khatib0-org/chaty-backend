use std::io::ErrorKind;
use std::sync::Arc;

use chaty_config::{config, Settings};
use chaty_database::{DatabaseInfoNoSql, DatabaseInfoSql, DatabaseNoSql, DatabaseSql};
use chaty_result::errors::{BoxedErr, ErrorType, SimpleError};
use prometheus::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::controller::{ApiController, ApiControllerArgs};
use crate::observability::MetricsCollector;

pub mod broker;
pub mod observability;

use broker::BrokerConfig;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApiServer {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerConfig>,
  pub(super) metrics_registry: Arc<Registry>,
  pub(super) metrics: Arc<MetricsCollector>,
}

impl ApiServer {
  pub async fn new() -> Result<ApiServer, BoxedErr> {
    let se = |err: BoxedErr, typ: ErrorType, msg: &str| {
      return SimpleError { err, err_type: typ, message: msg.to_string() };
    };

    ApiServer::setup_logging();
    let config = config().await;

    // Initialize observability
    let (metrics_registry, metrics) = observability::init_otel()?;

    let nosql_db = DatabaseInfoNoSql::ScyllaDb {
      uri: config.database.scylladb.clone(),
      keyspace: config.database.db_name.clone(),
    }
    .connect()
    .await
    .map_err(|err| {
      se(Box::new(std::io::Error::new(ErrorKind::NotConnected, err)), ErrorType::Connection, "")
    })?;

    let sql_db = DatabaseInfoSql::Postgres { dsn: config.database.postgres.clone() }
      .connect()
      .await
      .map_err(|err| {
        se(Box::new(std::io::Error::new(ErrorKind::NotConnected, err)), ErrorType::Connection, "")
      })?;

    // Initialize Redpanda broker connection
    let broker = BrokerConfig::new(&config)
      .await
      .map_err(|err| se(err, ErrorType::Connection, "failed to initialize broker"))?;

    let server = ApiServer {
      nosql_db: Arc::new(nosql_db),
      sql_db: Arc::new(sql_db),
      config: Arc::new(config),
      broker: Arc::new(broker),
      metrics_registry: Arc::new(metrics_registry),
      metrics: Arc::new(metrics),
    };

    Ok(server)
  }

  /// call the run of the grpc server
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let ctr_args = ApiControllerArgs {
      nosql_db: self.nosql_db.clone(),
      sql_db: self.sql_db.clone(),
      config: self.config.clone(),
      broker: self.broker.clone(),
      metrics_registry: self.metrics_registry.clone(),
      metrics: self.metrics.clone(),
    };

    let controller = ApiController::new(ctr_args);
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
