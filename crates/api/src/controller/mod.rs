pub(crate) mod audit;
mod groups;
mod router;
mod users;

use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};

use chaty_config::Settings;
use chaty_database::{DatabaseNoSql, DatabaseSql};
use chaty_proto::chaty_service_server::ChatyServiceServer;
use chaty_result::{errors::BoxedErr, middleware_context};
use reqwest::Client;
use tonic::{service::InterceptorLayer, transport::Server};
use tower::ServiceBuilder;
use tracing::info;

use crate::server::broker::BrokerApi;
use crate::server::observability::MetricsCollector;

pub struct ApiControllerArgs {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerApi>,
  pub(super) metrics: Arc<MetricsCollector>,
}

pub(crate) struct ApiController {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerApi>,
  pub(super) metrics: Arc<MetricsCollector>,
  pub(super) http_client: Arc<Client>,
}

impl ApiController {
  pub fn new(args: ApiControllerArgs) -> ApiController {
    let http_client = reqwest::Client::builder()
      .timeout(Duration::from_secs(10)) // Don't hang forever
      .connect_timeout(Duration::from_secs(3))
      .pool_idle_timeout(Duration::from_secs(90))
      .pool_max_idle_per_host(10) // Keep connections alive for reuse
      .build()
      .expect("Failed to create reqwest client for ApiController");

    let controller = ApiController {
      nosql_db: args.nosql_db,
      sql_db: args.sql_db,
      config: args.config,
      broker: args.broker,
      metrics: args.metrics,
      http_client: Arc::new(http_client),
    };

    controller
  }

  // run the grpc server
  pub async fn run(self) -> Result<(), BoxedErr> {
    let controller = ApiController { ..self };
    let url = controller.config.hosts.api.clone();

    let svc = ChatyServiceServer::new(controller);
    let layer_stack = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context));

    info!("the api server is listening on: {}", url);
    Server::builder()
      .layer(layer_stack)
      .add_service(svc)
      .serve(url.parse::<SocketAddr>().unwrap())
      .await?;

    Ok(())
  }
}
