use chaty_proto::{
  chaty_service_server::ChatyService, UserCreateRequest, UserCreateResponse, UsersLoginRequest,
  UsersLoginResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{users::users_create::users_create, ApiController};

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
    request: Request<UsersLoginRequest>,
  ) -> Result<Response<UsersLoginResponse>, Status> {
    todo!();
  }
}
