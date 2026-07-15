use std::sync::Arc;

use chaty_proto::{ServerBan, ServerMemberCompositeKey};
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};
use tonic::async_trait;

use crate::{ReferenceNoSqlDb, ServerBansRepository};

#[async_trait]
impl ServerBansRepository for ReferenceNoSqlDb {
  async fn server_bans_insert(&self, _ctx: Arc<Context>, ban: &ServerBan) -> Result<(), DBError> {
    let mut servers = self.server_bans.lock().await;

    let id =
      &ServerMemberCompositeKey { server_id: ban.server_id.clone(), user_id: ban.user_id.clone() };
    if servers.get(id).is_none() {
      servers.insert(id.clone(), ban.clone());
    }

    Ok(())
  }

  async fn server_bans_get(&self, server_id: &str, user_id: &str) -> Result<ServerBan, DBError> {
    let bans = self.server_bans.lock().await;

    let id =
      ServerMemberCompositeKey { server_id: server_id.to_string(), user_id: user_id.to_string() };

    bans.get(&id).cloned().ok_or_else(|| DBError {
      err_type: ErrorType::NotFound,
      msg: format!("server ban not found for server {} user {}", server_id, user_id),
      path: "database.server_bans.server_bans_get".to_string(),
      ..Default::default()
    })
  }
}
