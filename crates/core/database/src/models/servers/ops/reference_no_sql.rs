use async_trait::async_trait;
use chaty_proto::Server;
use chaty_result::errors::{DBError, ErrorType};

use crate::{ReferenceNoSqlDb, ServersRepository};

#[async_trait]
impl ServersRepository for ReferenceNoSqlDb {
  async fn servers_get_server_by_id(&self, server_id: &str) -> Result<Server, DBError> {
    let servers = self.servers.lock().await;

    servers.get(server_id).cloned().ok_or_else(|| DBError {
      err_type: ErrorType::NotFound,
      msg: format!("server {} not found", server_id),
      path: "database.servers.servers_get_server_by_id".to_string(),
      ..Default::default()
    })
  }
}
