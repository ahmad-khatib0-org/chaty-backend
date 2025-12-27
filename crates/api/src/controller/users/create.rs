use chaty_proto::{UserCreateRequest, UserCreateResponse};
use tonic::{Request, Response, Status};

use crate::controller::ApiController;

pub async fn users_create(
  ctr: &ApiController,
  request: Request<UserCreateRequest>,
) -> Result<Response<UserCreateResponse>, Status> {
  todo!();
}
