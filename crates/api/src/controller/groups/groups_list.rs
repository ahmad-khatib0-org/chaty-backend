use std::{sync::Arc, time::Instant};

use chaty_proto::{
  groups_list_response::Response::{Data, Error},
  GroupsListRequest, GroupsListResponse, GroupsListResponseData,
};
use chaty_result::{
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ERROR_ID_INTERNAL},
  network::{build_pagination_response, check_last_id},
};
use tonic::{Code, Request, Response, Status};

use crate::controller::ApiController;

pub async fn groups_list(
  ctr: &ApiController,
  request: Request<GroupsListRequest>,
) -> Result<Response<GroupsListResponse>, Status> {
  let start = Instant::now();
  ctr.metrics.record_groups_list_success();

  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.groups.groups_list";
  let req = request.into_inner();

  let return_err =
    |e: AppError| Response::new(GroupsListResponse { response: Some(Error(e.to_proto())) });

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if let Err(err) = check_last_id(ctx.clone(), path, &req.pagination) {
    ctr.metrics.record_groups_list_failure();
    return Ok(return_err(err));
  }

  let pagination = &req.pagination.as_ref().unwrap();
  let last_id = pagination.last_id();

  let db_start = Instant::now();
  ctr.metrics.record_db_operation("channels_groups_list");

  let db_res = ctr.nosql_db.channels_groups_list(ctx.clone(), last_id, 10).await;
  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("channels_groups_list", db_duration);

  if let Err(err) = db_res {
    ctr.metrics.record_db_error("channels_groups_list", &err.msg);
    ctr.metrics.record_groups_list_failure();
    let err_to_return = ie(Box::new(err));
    return Ok(return_err(err_to_return));
  }

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("groups.groups_list", request_duration);

  let groups = db_res.unwrap();
  let groups_count = groups.len();
  Ok(Response::new(GroupsListResponse {
    response: Some(Data(GroupsListResponseData {
      groups,
      pagination: Some(build_pagination_response(pagination, groups_count)),
    })),
  }))
}
