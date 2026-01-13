mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use async_trait::async_trait;
use chaty_result::errors::DBError;

#[async_trait]
pub trait ServerMembersRepository: Sync + Send {
  /// Get servers IDs for the specified user,
  async fn server_members_get_server_ids_by_user_id(
    &self,
    user_id: &str,
  ) -> Result<Vec<String>, DBError>;
}
