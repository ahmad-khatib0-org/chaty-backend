use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::Channel;
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{ChannelsRepository, ReferenceNoSqlDb};

#[async_trait()]
impl ChannelsRepository for ReferenceNoSqlDb {
  async fn channels_groups_create(
    &self,
    _ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    let mut channels = self.channels.lock().await;
    let path = "database.channels.channels_create".to_string();

    if channels.contains_key(&channel.id) {
      let msg = "channel already exists".to_string();
      Err(DBError { err_type: ErrorType::ResourceExists, msg, path, ..Default::default() })
    } else {
      channels.insert(channel.id.to_string(), channel.clone());
      Ok(())
    }
  }
}
