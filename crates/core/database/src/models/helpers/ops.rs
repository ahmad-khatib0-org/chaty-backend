mod reference_no_sql;
mod shared;

#[cfg(feature = "scylladb")]
mod scylladb;

use async_trait::async_trait;
use chaty_config::FeaturesLimits;
use chaty_proto::{Server, ServerMember};

#[async_trait]
pub trait HelpersRepository: Sync + Send {
  fn server_members_get_ranking(&self, member: &ServerMember, server: &Server) -> i64;
  /// Get limits for this user
  async fn users_get_limits(&self, user_id: &str) -> FeaturesLimits;
}
