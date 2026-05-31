use std::collections::HashSet;

use deadpool_redis::redis::AsyncCommands;

use crate::Cache;

pub static ONLINE_SET: &str = "online";

impl Cache {
  /// Check whether a set of users is online, returns a set of the online user IDs
  pub async fn presence_filter_online_users(&self, user_ids: &'_ [String]) -> HashSet<String> {
    let path = "cache.presence.presence_filter_online_users".to_string();
    let mut set = HashSet::new();
    if user_ids.is_empty() {
      return set;
    }

    // We need to handle a special case where only one is present
    // as for versions prior to 6.2.0., Redis does not like us sending
    // a list of just one ID to the server.
    if user_ids.len() == 1 {
      if self.presence_is_user_online(&user_ids[0]).await {
        set.insert(user_ids[0].to_string());
      }

      return set;
    }

    let conn = self.get_conn(&path).await;
    if conn.is_err() {
      return set;
    }
    let data: Vec<bool> = conn
      .unwrap()
      .smismember(ONLINE_SET, user_ids)
      .await
      .expect("should return online set of user ids");

    if data.is_empty() {
      return set;
    }

    for i in 0..user_ids.len() {
      if data[i] {
        set.insert(user_ids[i].to_string());
      }
    }

    set
  }

  pub async fn presence_is_user_online(&self, user_id: &str) -> bool {
    let path = "cache.presence.presence_is_user_online".to_string();
    let conn = self.get_conn(&path).await;
    if conn.is_err() {
      return false;
    }
    return conn.unwrap().exists(user_id).await.unwrap_or(false);
  }
}
