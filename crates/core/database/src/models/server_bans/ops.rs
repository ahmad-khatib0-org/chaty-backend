use std::sync::Arc;

use chaty_proto::ServerBan;
use chaty_result::{context::Context, errors::DBError};
use tonic::async_trait;

mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

#[async_trait]
pub trait ServerBansRepository: Sync + Send {
  /// Insert new ban into database
  async fn server_bans_insert(&self, ctx: Arc<Context>, ban: &ServerBan) -> Result<(), DBError>;

  /// Get a ban by server_id and user_id
  async fn server_bans_get(&self, server_id: &str, user_id: &str) -> Result<ServerBan, DBError>;
}
