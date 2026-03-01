use std::sync::Arc;

use chaty_proto::User;
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType, InternalError},
};
use deadpool_redis::redis::AsyncCommands;

use crate::Cache;

impl Cache {
  pub async fn users_get_by_id(&self, id: &str) -> Result<Option<User>, BoxedErr> {
    let path = "cache.users.users_get_by_id".to_string();
    let mut conn = self.get_conn(&path).await?;

    let ie = |err: BoxedErr, msg: &str| {
      InternalError::new(path.clone(), err, ErrorType::InternalError, false, msg.into())
    };

    let res: Option<String> = conn
      .get(self.users_by_id_key(id))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get user data from redis"))?;

    match res {
      Some(res) => {
        let user: User = serde_json::from_str(&res)
          .map_err(|err| ie(Box::new(err), "failed to serialize user "))?;
        Ok(Some(user))
      }
      None => Ok(None),
    }
  }

  pub async fn users_get_or_insert_by_id(
    &self,
    ctx: Arc<Context>,
    id: &str,
  ) -> Result<User, DBError> {
    let path = "cache.users.users_get_or_insert_by_id".to_string();
    let mut conn = self.get_conn(&path).await?;

    let ie =
      |err: BoxedErr, msg: &str| DBError::new(path.clone(), err, ErrorType::InternalError, msg);

    let res: Option<String> = conn
      .get(self.users_by_id_key(id))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get user data from redis"))?;

    match res {
      Some(res) => {
        let user: User = serde_json::from_str(&res)
          .map_err(|err| ie(Box::new(err), "failed to deserialize user"))?;
        Ok(user)
      }
      None => {
        let user = self.sql_db.users_get_by_id(ctx, id).await?;
        let mut conn = self.get_conn(&path).await?;

        let payload = serde_json::to_string(&user).map_err(|err| {
          let msg = "failed to serialize user".to_string();
          DBError::new(path.clone(), Box::new(err), ErrorType::JsonMarshal, msg)
        })?;

        let _: () = conn.set(self.users_by_id_key(id), payload).await.map_err(|err| {
          let msg = "failed to insert user in redis";
          DBError::new(&path, Box::new(err), ErrorType::DBInsertError, msg)
        })?;

        Ok(user)
      }
    }
  }
}
