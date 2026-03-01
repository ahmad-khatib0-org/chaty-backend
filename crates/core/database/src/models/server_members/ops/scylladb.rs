use std::io::{Error, ErrorKind};

use async_trait::async_trait;
use chaty_proto::{File, ServerMember};
use chaty_result::errors::{BoxedErr, DBError, ErrorType};
use chaty_utils::time::time_get_millis;

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

  async fn server_members_get_member(
    &self,
    server_id: &str,
    user_id: &str,
  ) -> Result<ServerMember, DBError> {
    let path = "database.server_members.server_members_get_member".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    let rows = self
      .db
      .execute_unpaged(&self.prepared.server_members.get_server_member_by_id, (server_id, user_id))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch member"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    let typed_rows = rows
        .rows::<(String, String, Option<File>, Option<String>, i64, Vec<String>, Option<i64>, bool, bool)>()
        .map_err(|e| de(Box::new(e), "failed to iterate over rows"))?;

    let mut row_iter = typed_rows;
    let row_result = row_iter.next().ok_or_else(|| {
      de(Box::new(Error::new(ErrorKind::NotFound, "member not found")), "member not found")
    })?;

    let (
      user_id_found,
      username,
      avatar,
      nickname,
      joined_at,
      roles,
      timeout,
      can_publish,
      can_receive,
    ) = row_result.map_err(|e| de(Box::new(e), "deserialization failed"))?;

    Ok(ServerMember {
      server_id: server_id.to_string(),
      user_id: user_id_found,
      username,
      avatar,
      nickname,
      joined_at,
      roles,
      timeout,
      can_publish,
      can_receive,
    })
  }

  fn server_members_is_member_in_timeout(&self, member: &ServerMember) -> bool {
    if let Some(timeout) = member.timeout {
      timeout > time_get_millis()
    } else {
      false
    }
  }
}
