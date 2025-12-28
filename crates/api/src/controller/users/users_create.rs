use std::sync::Arc;

use chaty_proto::{
  user_create_response::Response::{Data, Error},
  User, UserCreateRequest, UserCreateResponse, UserCreateResponseData,
};
use chaty_result::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ErrorType, ERROR_ID_INTERNAL},
};
use tonic::{Code, Request, Response, Status};
use validator::ValidateEmail;

use crate::controller::ApiController;

pub async fn users_create(
  ctr: &ApiController,
  request: Request<UserCreateRequest>,
) -> Result<Response<UserCreateResponse>, Status> {
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.users.users_create";
  let req = request.into_inner();

  let return_err =
    |e: AppError| Response::new(UserCreateResponse { response: Some(Error(e.to_proto())) });
  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = users_create_validate(ctx.clone(), path, &req) {
    return Ok(return_err(err));
  }

  let user = User { ..Default::default() };
  let db_res = ctr.sql_db.users_create(ctx.clone(), &user).await;
  if db_res.is_err() {
    let err = db_res.unwrap_err();
    if err.err_type == ErrorType::ResourceExists {
      // return Ok(return_err())
    } else {
      return Ok(return_err(ie(Box::new(err))));
    }
  }

  Ok(Response::new(UserCreateResponse { response: Some(Data(UserCreateResponseData {})) }))
}

fn users_create_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &UserCreateRequest,
) -> Result<(), AppError> {
  let ae = |id: &str| {
    return AppError::new(ctx, path, id, None, "", Code::InvalidArgument.into(), None);
  };

  if !req.email.validate_email() {
    return Err(ae("users.email.invalid"));
  }

  // check password : 8, have symbol, number, small and capital latter

  Ok(())
}
