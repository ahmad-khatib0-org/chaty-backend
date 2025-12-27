use std::io::ErrorKind;
use std::sync::Arc;

use chaty_config::config;
use chaty_database::{Database, DatabaseInfo};
use chaty_result::{BoxedErr, ErrorType, SimpleError};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::controller::{ApiController, ApiControllerArgs};

#[derive(Debug)]
pub struct ApiServer {
  pub(super) db: Arc<Database>,
}

impl ApiServer {
  pub async fn new() -> Result<ApiServer, BoxedErr> {
    let se = |err: BoxedErr, typ: ErrorType, msg: &str| {
      return SimpleError { err, _type: typ, message: msg.to_string() };
    };

    setup_logging();
    let config = config().await;
    let db =
      DatabaseInfo::ScyllaDb { uri: config.database.scylladb, keyspace: config.database.db_name }
        .connect()
        .await
        .map_err(|err| {
          se(Box::new(std::io::Error::new(ErrorKind::NotConnected, err)), ErrorType::Connection, "")
        });

    let server = ApiServer { db: Arc::new(db.unwrap()) };

    Ok(server)
  }

  /// call the run of the grpc server
  pub async fn run(&self) -> Result<(), BoxedErr> {
    let ctr_args = ApiControllerArgs { db: self.db.clone() };
    let controller = ApiController::new(ctr_args);

    controller.run().await?; // this will block
    Ok(())
  }
}

fn setup_logging() {
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

  let subscriber =
    tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer());

  tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
