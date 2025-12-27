use chaty_proto::{
  chaty_service_server::ChatyService, LoginRequest, LoginResponse, UserCreateRequest,
  UserCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{users::create::users_create, ApiController};

#[tonic::async_trait]
impl ChatyService for ApiController {
  async fn users_create(
    &self,
    request: Request<UserCreateRequest>,
  ) -> Result<Response<UserCreateResponse>, Status> {
    users_create(self, request).await
  }

  async fn users_login(
    &self,
    request: Request<LoginRequest>,
  ) -> Result<Response<LoginResponse>, Status> {
    todo!();
  }
}
