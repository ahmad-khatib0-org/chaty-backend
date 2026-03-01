use std::{sync::Arc, time::Instant};

use chaty_database::permissions::DatabasePermissionQuery;
use chaty_permission::{calculate_channel_permissions, ChannelPermission};
use chaty_proto::{
  channels_get_response::Response::{Data, Error},
  ChannelsGetRequest, ChannelsGetResponse,
};
use chaty_result::{
  context::Context,
  errors::{
    not_found_error, permission_denied_error, AppError, AppErrorErrors, BoxedErr, ErrorType,
    ERROR_ID_INTERNAL,
  },
};
use tonic::{Code, Request, Response, Status};

use crate::controller::ApiController;

pub async fn channels_get(
  ctr: &ApiController,
  request: Request<ChannelsGetRequest>,
) -> Result<Response<ChannelsGetResponse>, Status> {
  let start = Instant::now();
  ctr.metrics.record_channels_get_success();

  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.channels.channels_get";
  let req = request.into_inner();

  let return_err = |e: AppError| {
    ctr.metrics.record_channels_get_failure();
    Response::new(ChannelsGetResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  let cache_start = Instant::now();
  let user = ctr.cache.users_get_or_insert_by_id(ctx.clone(), ctx.session().user_id()).await;
  let cache_duration = cache_start.elapsed().as_secs_f64();
  ctr.metrics.observe_cache_operation_duration("users_get_or_insert_by_id", cache_duration);

  if user.is_err() {
    let err = user.unwrap_err();
    if err.err_type == ErrorType::NoRows {
      return Ok(return_err(not_found_error(ctx.clone(), path, Some(Box::new(err)))));
    } else {
      return Ok(return_err(ie(Box::new(err))));
    }
  }
  let user = user.unwrap();

  let cache_start = Instant::now();
  let chan = ctr.cache.channels_get_or_insert_by_id(ctx.clone(), &req.id).await;
  let cache_duration = cache_start.elapsed().as_secs_f64();
  ctr.metrics.observe_cache_operation_duration("channels_get_or_insert_by_id", cache_duration);

  if chan.is_err() {
    let err = chan.unwrap_err();
    if err.err_type == ErrorType::NoRows {
      return Ok(return_err(not_found_error(ctx.clone(), path, Some(Box::new(err)))));
    } else {
      return Ok(return_err(ie(Box::new(err))));
    }
  }
  let chan = chan.unwrap();

  let mut query =
    DatabasePermissionQuery::new(ctx.clone(), &ctr.sql_db, &ctr.nosql_db, &user).channel(&chan);
  let has_perm = calculate_channel_permissions(&mut query)
    .await
    .has_channel_permission(ChannelPermission::ViewChannel);

  if !has_perm {
    return Ok(return_err(permission_denied_error(ctx.clone(), path, None)));
  }

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("channels.channels_get", request_duration);

  Ok(Response::new(ChannelsGetResponse { response: Some(Data(chan)) }))
}
