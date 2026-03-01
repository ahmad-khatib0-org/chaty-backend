mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use async_trait::async_trait;
use chaty_proto::ServerMember;
use chaty_result::errors::DBError;

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
}
