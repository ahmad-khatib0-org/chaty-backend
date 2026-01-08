mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::Channel;
use chaty_result::{context::Context, errors::DBError};

#[async_trait]
pub trait ChannelsRepository: Sync + Send {
  /// Insert a channel into database
  async fn channels_groups_create(
    &self,
    ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError>;
}
