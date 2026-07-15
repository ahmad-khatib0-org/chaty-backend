use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use async_trait::async_trait;
use chaty_proto::{Channel, ChannelGroup, GroupsListItem};
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType},
};

use scylla::statement::batch::Batch;

use crate::{ChannelHelpers, ChannelsRepository, ScyllaDb};

#[async_trait()]
impl ChannelHelpers for ScyllaDb {
  async fn channels_insert_channel_and_channel_by_user(
    &self,
    path: &str,
    channel: &Channel,
    user_id: &str,
  ) -> Result<(), DBError> {
    let de = |err: BoxedErr, msg: &str| {
      let path = path.to_string();
      return DBError { path, err_type: ErrorType::DBInsertError, msg: msg.into(), err };
    };

    let created_at = channel.created_at;
    let updated_at = channel.updated_at;

    // Create a Logged Batch for atomic-like dual-write
    let mut batch = Batch::default();
    batch.append_statement(self.prepared.channels.insert_channel.clone());
    batch.append_statement(self.prepared.channels.insert_channel_by_user.clone());

    self
      .db
      .batch(
        &batch,
        (
          (
            &channel.id,
            &channel.channel_type,
            &channel.saved,
            &channel.direct,
            &channel.group,
            &channel.text,
            &created_at,
            &updated_at,
          ),
          (
            user_id,
            &channel.id,
            &channel.channel_type,
            &channel.saved,
            &channel.direct,
            &channel.group,
            &channel.text,
            &created_at,
            &updated_at,
          ),
        ),
      )
      .await
      .map_err(|err| de(Box::new(err), "failed to insert a channel and a channel_by_user"))?;

    Ok(())
  }
}

#[async_trait()]
impl ChannelsRepository for ScyllaDb {
  async fn channels_groups_create(
    &self,
    _ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    let path = "database.channels.channels_create".to_string();

    if &channel.channel_type != &"group".to_string() {
      let msg = "Channel must be a group type with valid group data".to_string();
      return Err(DBError { path, err_type: ErrorType::InvalidData, msg, ..Default::default() });
    }

    let group = channel.group.as_ref().unwrap();

    self.channels_insert_channel_and_channel_by_user(&path, channel, &group.user_id).await?;

    Ok(())
  }

  async fn channels_groups_list(
    &self,
    ctx: Arc<Context>,
    last_id: &str,
    limit: i32,
  ) -> Result<Vec<GroupsListItem>, DBError> {
    let path = "database.channels.channels_groups_list".to_string();
    let user_id = ctx.session.user_id();

    let de = |err: BoxedErr, msg: String, err_type: Option<ErrorType>| {
      let err_type = err_type.unwrap_or(ErrorType::DatabaseError);
      return DBError { path: path.clone(), err_type, msg, err };
    };

    let rows = if last_id.is_empty() {
      self
        .db
        .execute_unpaged(&self.prepared.channels.groups_list_first_page, (user_id, limit))
        .await
    } else {
      self
        .db
        .execute_unpaged(&self.prepared.channels.groups_list_next_page, (user_id, last_id, limit))
        .await
    }
    .map_err(|err| de(Box::new(err), format!("failed to fetch groups"), None))?
    .into_rows_result()
    .map_err(|err| de(Box::new(err), format!("failed to fetch groups"), None))?;

    let groups: Vec<GroupsListItem> = rows
      .rows::<(String, ChannelGroup, i64)>()
      .map_err(|err| de(Box::new(err), "failed to create iterator".to_string(), None))?
      .map(|row_result| {
        row_result
          .map(|(id, group, created_at)| GroupsListItem { id, group: Some(group), created_at })
          .map_err(|err| de(Box::new(err), "failed to deserialize row".to_string(), None))
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(groups)
  }

  async fn channels_get_by_id(
    &self,
    _ctx: Arc<Context>,
    channel_id: &str,
  ) -> Result<Channel, DBError> {
    let path = "database.channels.channels_get_by_id".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      DBError { path: path.clone(), err_type, msg: msg.to_string(), err }
    };

    let rows = self
      .db
      .execute_unpaged(&self.prepared.channels.get_channel_by_id, (channel_id,))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch channel data"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    let mut typed_rows =
      rows.rows::<Channel>().map_err(|e| de(Box::new(e), "failed to iterate over rows"))?;

    typed_rows
      .next()
      .ok_or_else(|| DBError {
        err_type: ErrorType::NoRows,
        err: Box::new(Error::new(ErrorKind::NotFound, "channel not found")),
        msg: "channel not found".to_string(),
        path: path.clone(),
      })?
      .map_err(|e| de(Box::new(e), "deserialization failed"))
  }

  async fn channels_get_channels_ids_by_user_id(
    &self,
    user_id: &str,
    channel_types: &[&str],
  ) -> Result<Vec<String>, DBError> {
    let path = "database.channels.channels_get_channels_ids_by_user_id".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      return DBError { path: path.clone(), err_type, msg: msg.to_string(), err };
    };

    // Build query with IN clause for multiple types
    let placeholders = channel_types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
      "SELECT channel_id FROM channels_by_recipient WHERE user_id = ? AND channel_type IN ({})",
      placeholders
    );

    let rows = self
      .db
      .query_unpaged(query, (user_id, channel_types))
      .await
      .map_err(|e| de(Box::new(e), "failed to fetch channel ids"))?
      .into_rows_result()
      .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

    rows
      .rows::<(String,)>()
      .map_err(|e| de(Box::new(e), "failed to iterate over rows"))?
      .map(|row_res| row_res.map(|(id,)| id).map_err(|e| de(Box::new(e), "deserialization failed")))
      .collect()
  }

  async fn channels_insert(&self, channel: &Channel, user_id: &str) -> Result<(), DBError> {
    let path = "database.channels.channels_insert";

    self.channels_insert_channel_and_channel_by_user(path, channel, user_id).await?;

    Ok(())
  }
}
