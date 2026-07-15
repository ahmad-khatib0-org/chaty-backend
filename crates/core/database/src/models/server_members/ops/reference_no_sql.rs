use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chaty_proto::ServerMember;
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};
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

  async fn server_members_get_by_ids(
    &self,
    server_id: &str,
    user_ids: &[String],
  ) -> Result<Vec<ServerMember>, DBError> {
    if user_ids.is_empty() {
      return Ok(vec![]);
    }

    let members = self.server_members.lock().await;

    let user_ids_set: HashSet<_> = user_ids.iter().collect();

    let found_members: Vec<ServerMember> = members
      .iter()
      .filter(|srv| srv.1.server_id == server_id && user_ids_set.contains(&srv.1.user_id))
      .map(|srv| srv.1.clone())
      .collect();

    Ok(found_members)
  }

  async fn server_members_insert(
    &self,
    _ctx: Arc<Context>,
    member: &ServerMember,
  ) -> Result<(), DBError> {
    let mut members = self.server_members.lock().await;

    let key = format!("{}:{}", member.server_id, member.user_id);
    if members.get(&key).is_none() {
      members.insert(key, member.clone());
    }

    Ok(())
  }

  async fn server_members_count_for_user(&self, user_id: &str) -> Result<i64, DBError> {
    let members = self.server_members.lock().await;

    let count: Vec<&String> =
      members.iter().filter(|srv| srv.1.user_id == user_id).map(|srv| srv.0).collect();

    Ok(count.len() as i64)
  }
}
