mod reference_no_sql;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::Message;
use chaty_result::{context::Context, errors::DBError};

use crate::models::messages::MessageTimePeriod;

#[async_trait]
pub trait MessagesRepository: Sync + Send {
  /// get messages by a given channel_id
  async fn messages_get_by_channel_id(
    &self,
    ctx: Arc<Context>,
    channel_id: String,
    limit: i32,
    time: MessageTimePeriod,
  ) -> Result<Vec<Message>, DBError>;
}
