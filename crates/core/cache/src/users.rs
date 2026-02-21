use chaty_proto::User;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
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
}
