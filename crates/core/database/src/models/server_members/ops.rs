mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::ServerMember;
use chaty_result::{context::Context, errors::DBError};

#[async_trait]
pub trait ServerMembersRepository: Sync + Send {
  /// Get servers IDs for the specified user,
  async fn server_members_get_server_ids_by_user_id(
    &self,
    user_id: &str,
  ) -> Result<Vec<String>, DBError>;

  /// Fetch a server member by their id
  async fn server_members_get_member(
    &self,
    server_id: &str,
    user_id: &str,
  ) -> Result<ServerMember, DBError>;

  /// Check whether this member is in timeout
  fn server_members_is_member_in_timeout(&self, member: &ServerMember) -> bool;

  async fn server_members_get_by_ids(
    &self,
    server_id: &str,
    user_ids: &[String],
  ) -> Result<Vec<ServerMember>, DBError>;

  async fn server_members_count_for_user(&self, user_id: &str) -> Result<i64, DBError>;

  /// Insert a new server member into database
  async fn server_members_insert(
    &self,
    ctx: Arc<Context>,
    member: &ServerMember,
  ) -> Result<(), DBError>;
}
