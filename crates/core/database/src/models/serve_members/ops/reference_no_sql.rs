use async_trait::async_trait;
use chaty_result::errors::DBError;

use crate::{ReferenceNoSqlDb, ServerMembersRepository};

#[async_trait]
impl ServerMembersRepository for ReferenceNoSqlDb {
  async fn server_members_get_server_ids_by_user_id(
    &self,
    user_id: &str,
  ) -> Result<Vec<String>, DBError> {
    let server_members = self.server_members.lock().await;
    let _path = "database.server_members.server_members_get_server_ids_by_user_id".to_string();

    let servers_ids: Vec<String> = server_members
      .iter()
      .filter(|srv| srv.1.user_id == user_id)
      .map(|srv| srv.0.to_string())
      .collect();

    Ok(servers_ids)
  }
}
