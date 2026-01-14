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
        let msg = "Channel must be a group type with valid group data".to_string();
        return Err(DBError { path, err_type: ErrorType::InvalidData, msg, ..Default::default() });
      }
    };

    let de = |err: BoxedErr, msg: &str| {
      let path = path.clone();
      return DBError { path, err_type: ErrorType::DBInsertError, msg: msg.into(), err };
    };

    let created_at = channel.created_at.as_ref().map(|ts| CqlTimestamp(ts.seconds * 1000));
    let updated_at = channel.updated_at.as_ref().map(|ts| CqlTimestamp(ts.seconds * 1000));

    // Create a Logged Batch for atomic-like dual-write
    let mut batch1 = Batch::default();
    batch1.append_statement(self.prepared.channels.insert_channel.clone());
    batch1.append_statement(self.prepared.channels.insert_channel_by_user.clone());

    let mut batch2 = Batch::default();
    batch2.append_statement(self.prepared.channels.insert_channel_by_recipient.clone());

    let recipient_params: Vec<_> = group
      .recipients
      .iter()
      .map(|recipient_id| (recipient_id, &channel.id, &channel.channel_type, &created_at))
      .collect();

    self
      .db
      .batch(
        &batch1,
        (
          (&channel.id, &channel.channel_type, group, &created_at, &updated_at),
          (&group.user_id, &channel.id, &channel.channel_type, group, &created_at, &updated_at),
        ),
      )
      .await
      .map_err(|err| de(Box::new(err), "failed to insert a channel, batch 1"))?;

    self
      .db
      .batch(&batch2, recipient_params)
      .await
      .map_err(|err| de(Box::new(err), "failed to create group (batch2 recipients)"))?;

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
      .rows::<(String, ChannelGroupDB, CqlTimestamp)>()
      .map_err(|err| de(Box::new(err), "failed to create iterator".to_string(), None))?
      .map(|row_result| {
        row_result
          .map(|(id, group_db, created_at)| {
            let group: ChannelGroup = group_db.into();
            GroupsListItem { id, group: Some(group), created_at: created_at.0 }
          })
          .map_err(|err| de(Box::new(err), "failed to deserialize row".to_string(), None))
      })
      .collect::<Result<Vec<_>, _>>()?;

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
      "SELECT channel_id FROM channels_by_recipient WHERE recipient_user_id = ? AND channel_type IN ({})",
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
