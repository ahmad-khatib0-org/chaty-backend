use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use chaty_utils::time::time_get_seconds;
use deadpool_redis::redis::AsyncCommands;

use crate::models::network::CachedTokenStatus;

use super::redis::{DefaultRedisClient, RedisCheck, RedisClient};

/// returns redis key
///
/// * `jti`: is jwt token id
pub fn auth_token_status_key(jti: &str) -> String {
  return format!("auth:token#{}", jti);
}

pub(super) async fn check_token(r: &DefaultRedisClient, jti: &str) -> Result<RedisCheck, BoxedErr> {
  let res = r.get_token(jti, "auth.controller.check_token").await?;

  match res {
    Some(status) => {
      if status.revoked {
        return Ok(RedisCheck::Revoked("token got revoked".into()));
      }

      Ok(RedisCheck::Allowed { status: Some(status) })
    }
    None => Ok(RedisCheck::Allowed { status: None }),
  }
}

// TODO: GET THE user device id
pub(super) async fn revoke_token(r: &DefaultRedisClient, jti: &str) -> Result<(), BoxedErr> {
  let path = "auth.controller.check_token";
  let res = r.get_token(&jti, path).await?;

  // This won't happen at all, unless the authn filter on envoy got removed
  // So this can happen in this case: only if the user didn't use the token
  // at all, so E,g the user logged in, and immediately closed the website
  // and didn't hit envoy once again, this extreme case, and mostly won't happen
  if res.is_none() {
    let last_checked = time_get_seconds() as i64;
    let payload = CachedTokenStatus { revoked: true, last_checked, dev_id: "".into() };
    return Ok(r.set_token(jti, &payload, &path).await?);
  }

  let mut payload = res.unwrap();
  payload.revoked = true;
  r.set_token(jti, &payload, path).await?;

  Ok(())
}

// TODO: get the device id
pub(super) async fn mark_checked_ok(r: &DefaultRedisClient, jti: &str) -> Result<(), BoxedErr> {
  let path = "auth.controller.mark_checked_ok";
  let res = r.get_token(&jti, path).await?;

  if res.is_none() {
    let payload = CachedTokenStatus {
      revoked: false,
      last_checked: time_get_seconds() as i64,
      dev_id: "".into(),
    };
    r.set_token(jti, &payload, &path).await?;
    return Ok(());
  }

  let mut payload = res.unwrap();
  payload.revoked = false;
  payload.last_checked = time_get_seconds() as i64;
  r.set_token(jti, &payload, &path).await?;

  Ok(())
}

pub async fn get_token(
  r: &DefaultRedisClient,
  token: &str,
  path: &str,
) -> Result<Option<CachedTokenStatus>, BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    InternalError::new(path.into(), err, ErrorType::InternalError, false, msg.into())
  };

  let mut con = r.get_conn(path).await?;
  let res: Option<String> = con
    .get(auth_token_status_key(token))
    .await
    .map_err(|err| ie(Box::new(err), "failed to get token data from redis"))?;

  match res {
    Some(json_str) => {
      let token_status: CachedTokenStatus = serde_json::from_str(&json_str)
        .map_err(|err| ie(Box::new(err), "failed to deserialize CachedTokenStatus"))?;
      Ok(Some(token_status))
    }
    None => Ok(None),
  }
}

pub async fn set_token(
  r: &DefaultRedisClient,
  jti: &str,
  data: &CachedTokenStatus,
  path: &str,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    InternalError::new(path.into(), err, ErrorType::InternalError, false, msg.into())
  };

  let mut con = r.get_conn(path).await?;
  let value = serde_json::to_string(data)
    .map_err(|err| ie(Box::new(err), "failed to serialize CachedTokenStatus"))?;

  let _: () = con
    .set(auth_token_status_key(jti), value)
    .await
    .map_err(|err| ie(Box::new(err), "failed to set CachedTokenStatus in redis"))?;

  Ok(())
}
