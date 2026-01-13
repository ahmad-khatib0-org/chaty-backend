mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{Channel, GroupsListItem};
use chaty_result::{context::Context, errors::DBError};

#[async_trait]
pub trait ChannelsRepository: Sync + Send {
  /// Insert a channel into database
  async fn channels_groups_create(
    &self,
    ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError>;

  /// List groups for the authenticated user with cursor pagination
  async fn channels_groups_list(
    &self,
    ctx: Arc<Context>,
    last_id: &str,
    limit: i32,
  ) -> Result<Vec<GroupsListItem>, DBError>;

  /// Get channel IDs for the specified user, filtered by channel types.
  async fn channels_get_channels_ids_by_user_id(
    &self,
    user_id: &str,
    channel_types: &[&str], // e.g., ["group", "direct_message"]
  ) -> Result<Vec<String>, DBError>;
}
