pub mod broker;
pub mod email;
pub mod observability;

use std::io::ErrorKind;
use std::sync::Arc;

use crate::controller::{ApiController, ApiControllerArgs};
use crate::email::{create_email_service, EmailService};
use crate::observability::{MetricsCollector, MetricsCollectorArgs};
use crate::worker::{WorkerApi, WorkerApiArgs};
use chaty_config::{config, Settings};
use chaty_database::{DatabaseInfoNoSql, DatabaseInfoSql, DatabaseNoSql, DatabaseSql};
use chaty_result::errors::{BoxedErr, ErrorType, SimpleError};
use chaty_result::translations_init;
use tokio::spawn;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use broker::BrokerApi;

pub struct ApiServer {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerApi>,
  pub(super) email_service: Arc<dyn EmailService>,
  pub(super) metrics: Arc<MetricsCollector>,
  pub(super) worker: Arc<WorkerApi>,
}

impl ApiServer {
  pub async fn new() -> Result<ApiServer, BoxedErr> {
    let se = |err: BoxedErr, typ: ErrorType, msg: &str| {
      return SimpleError { err, err_type: typ, message: msg.to_string() };
    };

    ApiServer::setup_logging();
    let config = config().await;

    translations_init(10, config.default_language.clone(), config.available_languages.clone())
      .map_err(|err| se(Box::new(err.clone()), ErrorType::InternalError, &err.to_string()))?;

    // Initialize observability
    let metrics = MetricsCollector::new(MetricsCollectorArgs { config: Arc::new(config.clone()) })?;

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
    let broker = BrokerApi::new(&config)
      .await
      .map_err(|err| se(err, ErrorType::Connection, "failed to initialize broker"))?;

    // Initialize email service
    let email_service = create_email_service(&config)
      .map_err(|err| se(err, ErrorType::ConfigError, "failed to initialize email service"))?;

    let worker = WorkerApi::new(WorkerApiArgs {
      config: Arc::new(config.clone()),
      email_service: email_service.clone(),
    })
    .await?;

    let server = ApiServer {
      nosql_db: Arc::new(nosql_db),
      sql_db: Arc::new(sql_db),
      config: Arc::new(config),
      broker: Arc::new(broker),
      worker: Arc::new(worker),
      email_service,
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
      metrics: self.metrics.clone(),
    };

    let worker_clone = self.worker.clone();
    spawn(async move {
      if let Err(e) = worker_clone.start().await {
        error!("Worker loop crashed: {:?}", e);
      }
    });

    let metrics_clone = self.metrics.clone();
    spawn(async move {
      if let Err(e) = metrics_clone.run().await {
        error!("Metrics server failed: {:?}", e);
      }
    });

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
