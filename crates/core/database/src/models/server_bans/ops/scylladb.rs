use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use chaty_proto::ServerBan;
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType},
};
use tonic::async_trait;

use crate::{ScyllaDb, ServerBansRepository};

#[async_trait]
impl ServerBansRepository for ScyllaDb {
  async fn server_bans_insert(&self, _ctx: Arc<Context>, ban: &ServerBan) -> Result<(), DBError> {
    let path = "database.servers.insert_server".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBInsertError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    self
      .db
      .execute_unpaged(&self.prepared.server_bans.insert_server_ban, ban)
      .await
      .map_err(|e| de(Box::new(e), "failed to insert server"))?;

    Ok(())
  }

  async fn server_bans_get(&self, server_id: &str, user_id: &str) -> Result<ServerBan, DBError> {
    let path = "database.server_bans.server_bans_get".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    let rows = self
      .db
      .execute_unpaged(&self.prepared.server_bans.get_server_ban, (server_id, user_id))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch server ban"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    let typed_rows =
      rows.rows::<ServerBan>().map_err(|e| de(Box::new(e), "failed to iterate over rows"))?;

    let mut row_iter = typed_rows;
    let ban = row_iter
      .next()
      .ok_or_else(|| {
        de(
          Box::new(Error::new(ErrorKind::NotFound, "server ban not found")),
          "server ban not found",
        )
      })?
      .map_err(|e| de(Box::new(e), "deserialization failed"))?;

    Ok(ban)
  }
}
