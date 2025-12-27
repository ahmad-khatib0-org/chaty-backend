mod router;
mod users;

use std::sync::Arc;

use chaty_database::Database;
use chaty_result::BoxedErr;

pub struct ApiControllerArgs {
  pub(super) db: Arc<Database>,
}

pub(crate) struct ApiController {
  pub(super) db: Arc<Database>,
}

impl ApiController {
  pub fn new(args: ApiControllerArgs) -> ApiController {
    let controller = ApiController { db: args.db };

    controller
  }

  // run the grpc server
  pub async fn run(&self) -> Result<(), BoxedErr> {
    Ok(())
  }
}
