mod router;
mod users;

use std::{net::SocketAddr, sync::Arc};

use chaty_config::Settings;
use chaty_database::Database;
use chaty_proto::chaty_service_server::ChatyServiceServer;
use chaty_result::{errors::BoxedErr, middleware_context};
use tonic::{service::InterceptorLayer, transport::Server};
use tower::ServiceBuilder;

pub struct ApiControllerArgs {
  pub(super) db: Arc<Database>,
  pub(super) config: Arc<Settings>,
}

pub(crate) struct ApiController {
  pub(super) db: Arc<Database>,
  pub(super) config: Arc<Settings>,
}

impl ApiController {
  pub fn new(args: ApiControllerArgs) -> ApiController {
    let controller = ApiController { db: args.db, config: args.config };

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
