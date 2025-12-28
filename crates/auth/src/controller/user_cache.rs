use std::sync::Arc;

use chaty_database::CachedUserData;
use chaty_result::{
  context::Context,
  errors::{BoxedErr, ErrorType, InternalError},
};
use deadpool_redis::redis::AsyncCommands;
use serde_json::to_string;

use super::Controller;

impl Controller {
  /// returns redis key
  ///
  /// * `email`: is the user email
  fn auth_user_data_key(email: &str) -> String {
    return format!("auth:user#{}", email);
  }

  pub async fn insert_auth_cached_user_data(
    &self,
    ctx: Arc<Context>,
    email: &str,
  ) -> Result<CachedUserData, BoxedErr> {
    let path = "auth.controller.insert_auth_cached_user_data";
    let ie = |err: BoxedErr, msg: &str| InternalError {
      err,
      msg: msg.into(),
      temp: true,
      path: path.into(),
      err_type: ErrorType::InternalError,
    };

    let data = self
      .store
      .clone()
      .users_get_auth_data(ctx, email)
      .await
      .map_err(|err| ie(Box::new(err), "failed to get user auth data"))?;

    let mut con = self.redis.get_conn(&path).await?;

    let payload =
      to_string(&data).map_err(|err| ie(Box::new(err), "failed to serialize CachedUserData"))?;

    let _: () = con
      .set(Controller::auth_user_data_key(email), payload)
      .await
      .map_err(|err| ie(Box::new(err), "failed to set CachedUserStatus in redis"))?;

    Ok(data)
  }

  pub async fn get_auth_cached_user_data(
    &self,
    email: &str,
  ) -> Result<Option<CachedUserData>, BoxedErr> {
    let path = "auth.controller.get_auth_cached_user_data";
    let ie = |err: BoxedErr, msg: &str| InternalError {
      err,
      msg: msg.into(),
      temp: true,
      path: path.into(),
      err_type: ErrorType::InternalError,
    };

    let mut con = self.redis.get_conn(&path).await?;
    let res: Option<String> = con
      .get(Controller::auth_user_data_key(email))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get user data from redis"))?;

    match res {
      Some(json_str) => {
        let data: CachedUserData = serde_json::from_str(&json_str)
          .map_err(|err| ie(Box::new(err), "failed to deserialize CachedUserData"))?;
        Ok(Some(data))
      }
      None => Ok(None),
    }
  }

  pub async fn get_or_insert_auth_cached_user_data(
    &self,
    ctx: Arc<Context>,
    email: &str,
  ) -> Result<CachedUserData, BoxedErr> {
    let user = self.get_auth_cached_user_data(email).await?;
    match user {
      Some(user) => Ok(user),
      None => self.insert_auth_cached_user_data(ctx, email).await,
    }
  }
}
