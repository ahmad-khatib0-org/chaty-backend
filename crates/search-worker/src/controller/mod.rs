use std::{sync::Arc, time::Duration};

use chaty_config::Settings;
use chaty_database::DatabaseSql;
use chaty_result::errors::BoxedErr;
use reqwest::Client;

use crate::server::observability::MetricsCollector;

pub struct SearchWorkerControllerArgs {
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) metrics: Arc<MetricsCollector>,
}

pub(crate) struct SearchWorkerController {
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) metrics: Arc<MetricsCollector>,
  pub(super) http_client: Arc<Client>,
}

impl SearchWorkerController {
  pub fn new(args: SearchWorkerControllerArgs) -> SearchWorkerController {
    let http_client = reqwest::Client::builder()
      .timeout(Duration::from_secs(10)) // Don't hang forever
      .connect_timeout(Duration::from_secs(3))
      .pool_idle_timeout(Duration::from_secs(90))
      .pool_max_idle_per_host(10) // Keep connections alive for reuse
      .build()
      .expect("Failed to create reqwest client for ApiController");

    let controller = SearchWorkerController {
      sql_db: args.sql_db,
      config: args.config,
      metrics: args.metrics,
      http_client: Arc::new(http_client),
    };

    controller
  }

  // run the worker service
  pub async fn run(self) -> Result<(), BoxedErr> {
    Ok(())
  }
}
