use std::io::{Error, ErrorKind};

use async_trait::async_trait;
use chaty_proto::Server;
use chaty_result::errors::{BoxedErr, DBError, ErrorType};

use crate::{ScyllaDb, ServersRepository};

#[async_trait]
impl ServersRepository for ScyllaDb {
  async fn servers_get_server_by_id(&self, server_id: &str) -> Result<Server, DBError> {
    let path = "database.servers.servers_get_server_by_id".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    let rows = self
      .db
      .execute_unpaged(&self.prepared.servers.get_server_by_id, (server_id,))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch server"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    let typed_rows =
      rows.rows::<Server>().map_err(|e| de(Box::new(e), "failed to iterate over rows"))?;

    let mut row_iter = typed_rows;
    let server_db = row_iter
      .next()
      .ok_or_else(|| {
        de(Box::new(Error::new(ErrorKind::NotFound, "server not found")), "server not found")
      })?
      .map_err(|e| de(Box::new(e), "deserialization failed"))?;

    Ok(server_db)
  }
}
