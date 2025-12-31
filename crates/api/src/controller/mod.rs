pub(crate) mod audit;
mod router;
mod users;

use std::{net::SocketAddr, sync::Arc};

use chaty_config::Settings;
use chaty_database::{DatabaseNoSql, DatabaseSql};
use chaty_proto::chaty_service_server::ChatyServiceServer;
use chaty_result::{errors::BoxedErr, middleware_context};
use tonic::{service::InterceptorLayer, transport::Server};
use tower::ServiceBuilder;

use crate::server::broker::BrokerConfig;
use crate::server::observability::MetricsCollector;
use prometheus::Registry;

pub struct ApiControllerArgs {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerConfig>,
  pub(super) metrics_registry: Arc<Registry>,
  pub(super) metrics: Arc<MetricsCollector>,
}

pub(crate) struct ApiController {
  pub(super) nosql_db: Arc<DatabaseNoSql>,
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) broker: Arc<BrokerConfig>,
  pub(super) metrics_registry: Arc<Registry>,
  pub(super) metrics: Arc<MetricsCollector>,
}

impl ApiController {
  pub fn new(args: ApiControllerArgs) -> ApiController {
    let controller = ApiController {
      nosql_db: args.nosql_db,
      sql_db: args.sql_db,
      config: args.config,
      broker: args.broker,
      metrics_registry: args.metrics_registry,
      metrics: args.metrics,
    };

    controller
  }

  // run the grpc server
  pub async fn run(self) -> Result<(), BoxedErr> {
    let controller = ApiController { ..self };
    let url = controller.config.hosts.api.clone();

    let svc = ChatyServiceServer::new(controller);
    let layer_stack = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context));

    Server::builder()
      .layer(layer_stack)
      .add_service(svc)
      .serve(url.parse::<SocketAddr>().unwrap())
      .await?;

    Ok(())
  }
}
