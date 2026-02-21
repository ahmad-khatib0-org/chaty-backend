use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::Connection;

use crate::Cache;

impl Cache {
  pub async fn get_conn(&self, path: &str) -> Result<Connection, BoxedErr> {
    let conn = self.redis.get().await.map_err(|err| {
      let msg = "failed to get a redis connection from pool".to_string();
      InternalError::new(path.into(), Box::new(err), ErrorType::InternalError, false, msg)
    })?;

    Ok(conn)
  }

  pub fn users_by_id_key(&self, user_id: &str) -> String {
    format!("users#data:{}", user_id)
  }
}
