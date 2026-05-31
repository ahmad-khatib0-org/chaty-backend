use std::{collections::HashSet, sync::Arc, time::Instant};

use chaty_database::{
  get_message_system_user_ids, permissions::DatabasePermissionQuery, MessageTimePeriod, UserApiExt,
};
use chaty_permission::{calculate_channel_permissions, ChannelPermission};
use chaty_proto::{
  messages_get_response::Response::{Data, Error},
  ApiUser, MessagesGetRequest, MessagesGetResponse, MessagesGetResponseData,
};
use chaty_result::{
  context::Context,
  errors::{
    not_found_error, permission_denied_error, AppError, AppErrorErrors, BoxedErr, ErrorType,
    ERROR_ID_INTERNAL,
  },
};
use futures::future::join_all;
use tonic::{Code, Request, Response, Status};

use crate::controller::ApiController;

pub async fn messages_get(
  ctr: &ApiController,
  request: Request<MessagesGetRequest>,
) -> Result<Response<MessagesGetResponse>, Status> {
  let start = Instant::now();

  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.messages.messages_get";
  let req = request.into_inner();

  let return_err = |e: AppError| {
    ctr.metrics.record_messages_get_failure();
    Response::new(MessagesGetResponse { response: Some(Error(e.to_proto())) })
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
  let chan = ctr.cache.channels_get_or_insert_by_id(ctx.clone(), &req.channel_id).await;
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
  if !calculate_channel_permissions(&mut query)
    .await
    .has_channel_permission(ChannelPermission::ReadMessageHistory)
  {
    return Ok(return_err(permission_denied_error(ctx.clone(), path, None)));
  }

  let time = match req.nearby {
    Some(ref nearby) => MessageTimePeriod::Relative { nearby: nearby.clone() },
    None => MessageTimePeriod::Absolute {
      before: req.before.clone(),
      after: req.after.clone(),
      sort: Some(req.sort()),
    },
  };

  let messages =
    ctr.nosql_db.messages_get_by_channel_id(ctx.clone(), req.channel_id.clone(), 50, time).await;
  if messages.is_err() {
    return Ok(return_err(ie(Box::new(messages.unwrap_err()))));
  }
  let messages = messages.unwrap();

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("messages.messages_get", request_duration);

  if req.include_users() {
    let user_ids = messages
      .iter()
      .flat_map(|msg| {
        let mut users = vec![msg.author_id.clone()];
        users.extend(get_message_system_user_ids(msg.system.as_ref()));
        users
      })
      .collect::<HashSet<String>>()
      .into_iter()
      .collect::<Vec<String>>();

    let online_user_ids = ctr.cache.presence_filter_online_users(&user_ids).await;

    let users: Vec<ApiUser> = join_all(
      online_user_ids
        .iter()
        .map(|user_id| ctr.cache.users_get_or_insert_by_id(ctx.clone(), &user_id))
        .map(|usr| async {
          let usr = usr.await.unwrap_or_default();
          let is_online = online_user_ids.contains(&user.id);
          usr.into_user_api(Some(&user), is_online).await
        }),
    )
    .await;

    ctr.metrics.record_messages_get_success();
    return Ok(Response::new(MessagesGetResponse {
      response: Some(Data(MessagesGetResponseData {
        messages,
        users,
        members: if let Some(chan_text) = chan.text {
          // TODO: consider caching if applicable!
          let result =
            ctr.nosql_db.server_members_get_by_ids(&chan_text.server_id, &user_ids).await;
          if result.is_err() {
            return Ok(return_err(ie(Box::new(result.unwrap_err()))));
          }
          result.unwrap()
        } else {
          vec![]
        },
      })),
    }));
  }

  ctr.metrics.record_messages_get_success();
  Ok(Response::new(MessagesGetResponse {
    response: Some(Data(MessagesGetResponseData { messages, ..Default::default() })),
  }))
}
