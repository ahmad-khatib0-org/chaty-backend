use chaty_proto::{
  chaty_service_server::ChatyService, UsersCreateRequest, UsersCreateResponse, UsersLoginRequest,
  UsersLoginResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{
  users::{users_create::users_create, users_login::users_login},
  ApiController,
};

#[tonic::async_trait]
impl ChatyService for ApiController {
  async fn users_create(
    &self,
    request: Request<UsersCreateRequest>,
  ) -> Result<Response<UsersCreateResponse>, Status> {
    users_create(self, request).await
  }

  async fn users_login(
    &self,
    request: Request<UsersLoginRequest>,
  ) -> Result<Response<UsersLoginResponse>, Status> {
    users_login(self, request).await
  }
}
