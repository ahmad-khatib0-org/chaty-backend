use async_trait::async_trait;
use chaty_result::errors::{BoxedErr, DBError, ErrorType};

use crate::{ScyllaDb, ServerMembersRepository};

#[async_trait]
impl ServerMembersRepository for ScyllaDb {
  async fn server_members_get_server_ids_by_user_id(
    &self,
    user_id: &str,
  ) -> Result<Vec<String>, DBError> {
    let path = "database.server_members.server_members_get_server_ids_by_user_id".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    let rows = self
      .db
      .execute_unpaged(&self.prepared.server_members.get_server_ids_by_user_id, (user_id,))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch server ids"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    rows
      .rows::<(String,)>()
      .map_err(|e| de(Box::new(e), "failed to iterate over rows"))?
      .map(|row_res| row_res.map(|(id,)| id).map_err(|e| de(Box::new(e), "deserialization failed")))
      .collect()
  }
}
