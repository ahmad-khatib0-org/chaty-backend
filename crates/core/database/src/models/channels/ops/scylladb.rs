use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{channel::ChannelData, Channel};
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{ChannelsRepository, ScyllaDb};

#[async_trait()]
impl ChannelsRepository for ScyllaDb {
  async fn channels_groups_create(
    &self,
    _ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    let path = "database.channels.channels_create".to_string();

    // Extract the group data from the oneof channel_data
    let group = match &channel.channel_data {
      Some(ChannelData::Group(g)) => g,
      _ => {
        return Err(DBError {
          err_type: ErrorType::InvalidData,
          msg: "Channel must be a group type with valid group data".to_string(),
          path,
          ..Default::default()
        });
      }
    };

    // Convert timestamps from prost Timestamp to i64 (milliseconds since epoch)
    let created_at = channel.created_at.as_ref().map(|ts| ts.seconds * 1000);
    let updated_at = channel.updated_at.as_ref().map(|ts| ts.seconds * 1000);

    let query =
      "INSERT INTO channels (id, channel_type, group, created_at, updated_at) VALUES (?, ?, ?, ?, ?)";

    let session = self.db();

    // Execute the query with parameters
    match session
      .query_unpaged(query, (&channel.id, &channel.channel_type, &group, &created_at, &updated_at))
      .await
    {
      Ok(_) => Ok(()),
      Err(err) => {
        let msg = format!("failed to create channel: {}", err);
        Err(DBError { err_type: ErrorType::DatabaseError, msg, path, ..Default::default() })
      }
    }
  }
}
