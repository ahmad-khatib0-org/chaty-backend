use chaty_result::errors::{DBError, ErrorType};
use deadpool_redis::Connection;

use crate::Cache;

impl Cache {
  pub async fn get_conn(&self, path: &str) -> Result<Connection, DBError> {
    let conn = self.redis.get().await.map_err(|err| {
      let msg = "failed to get a redis connection from pool".to_string();
      DBError::new(path, Box::new(err), ErrorType::InternalError, msg)
    })?;

    Ok(conn)
  }

  pub fn users_by_id_key(&self, user_id: &str) -> String {
    format!("users#data:{}", user_id)
  }

  pub fn channels_by_id_key(&self, channel_id: &str) -> String {
    format!("channels#data:{}", channel_id)
  }
}
