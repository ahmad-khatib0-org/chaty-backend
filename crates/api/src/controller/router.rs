use chaty_proto::{
  chaty_service_server::ChatyService, UsersCreateRequest, UsersCreateResponse,
  UsersEmailConfirmationRequest, UsersEmailConfirmationResponse, UsersLoginRequest,
  UsersLoginResponse, UsersForgotPasswordRequest, UsersForgotPasswordResponse,
};
use tonic::{Request, Response, Status};

use crate::controller::{
  users::{
    users_create::users_create, users_email_confirmation::users_email_confirmation,
    users_login::users_login, users_forgot_password::users_forgot_password,
  },
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

  async fn users_email_confirmation(
    &self,
    request: Request<UsersEmailConfirmationRequest>,
  ) -> Result<Response<UsersEmailConfirmationResponse>, Status> {
    users_email_confirmation(self, request).await
  }

  async fn users_forgot_password(
    &self,
    request: Request<UsersForgotPasswordRequest>,
  ) -> Result<Response<UsersForgotPasswordResponse>, Status> {
    users_forgot_password(self, request).await
  }
}
