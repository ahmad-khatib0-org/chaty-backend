use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{channel::ChannelData, Channel, ChannelGroup, GroupsListItem};
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType},
};

use scylla::{statement::batch::Batch, value::CqlTimestamp};

use crate::{models::channels::models::ChannelGroupDB, ChannelsRepository, ScyllaDb};

#[async_trait()]
impl ChannelsRepository for ScyllaDb {
  async fn channels_groups_create(
    &self,
    _ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    let path = "database.channels.channels_create".to_string();

    let group = match &channel.channel_data {
      Some(ChannelData::Group(g)) => g,
      _ => {
        return Err(DBError {
          path,
          err_type: ErrorType::InvalidData,
          msg: "Channel must be a group type with valid group data".to_string(),
          ..Default::default()
        });
      }
    };

    let created_at = channel.created_at.as_ref().map(|ts| CqlTimestamp(ts.seconds * 1000));
    let updated_at = channel.updated_at.as_ref().map(|ts| CqlTimestamp(ts.seconds * 1000));

    // Create a Logged Batch for atomic-like dual-write
    let mut batch = Batch::default();
    batch.append_statement(self.prepared.channels.insert_channel.clone());
    batch.append_statement(self.prepared.channels.insert_channel_by_user.clone());

    self
      .db
      .batch(
        &batch,
        (
          (&channel.id, &channel.channel_type, group, &created_at, &updated_at),
          (&group.user_id, &channel.id, &channel.channel_type, group, &created_at, &updated_at),
        ),
      )
      .await
      .map_err(|err| DBError {
        path,
        err_type: ErrorType::DatabaseError,
        msg: format!("failed to create group (batch): {}", err),
        err: Box::new(err),
      })?;

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

    let mut groups = Vec::new();
    rows
      .rows::<(String, ChannelGroupDB, CqlTimestamp)>()
      .map_err(|err| de(Box::new(err), format!("failed to iterate over channels groups"), None))?
      .map(|row| {
        row
          .map(|r| {
            let group: Option<ChannelGroup> = Some(r.1.into());
            groups.push(GroupsListItem { id: r.0, group, created_at: r.2 .0 });
          })
          .map_err(|err| de(Box::new(err), format!("failed to deserialize a channel group"), None))
      });

    Ok(groups)
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
      "SELECT channel_id FROM channels_by_user WHERE user_id = ? AND channel_type IN ({})",
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
}
