mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use async_trait::async_trait;
use chaty_proto::Server;
use chaty_result::errors::DBError;

#[async_trait]
pub trait ServersRepository: Sync + Send {
  /// Get a server by a specified server_id
  async fn servers_get_server_by_id(&self, server_id: &str) -> Result<Server, DBError>;
}
