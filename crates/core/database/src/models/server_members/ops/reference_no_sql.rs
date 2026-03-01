use async_trait::async_trait;
use chaty_proto::ServerMember;
use chaty_result::errors::{DBError, ErrorType};
use chaty_utils::time::time_get_millis;

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

  async fn server_members_get_member(
    &self,
    server_id: &str,
    user_id: &str,
  ) -> Result<ServerMember, DBError> {
    let members = self.server_members.lock().await;
    let path = "database.server_members.server_members_get_member".to_string();

    let member =
      members.iter().find(|srv| srv.1.user_id == user_id && srv.1.server_id == server_id);
    if member.is_some() {
      Ok(member.unwrap().1.clone())
    } else {
      let msg = "server member is not found".to_string();
      Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
    }
  }

  fn server_members_is_member_in_timeout(&self, member: &ServerMember) -> bool {
    if let Some(timeout) = member.timeout {
      timeout > time_get_millis()
    } else {
      false
    }
  }
}
