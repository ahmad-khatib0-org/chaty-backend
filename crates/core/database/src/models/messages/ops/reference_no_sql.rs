use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{Message, MessageSort};
use chaty_result::{context::Context, errors::DBError};

use crate::{
  models::messages::{MessageTimePeriod, MessagesRepository},
  ReferenceNoSqlDb,
};

#[async_trait]
impl MessagesRepository for ReferenceNoSqlDb {
  async fn messages_get_by_channel_id(
    &self,
    _ctx: Arc<Context>,
    channel_id: String,
    limit: i32,
    time: MessageTimePeriod,
  ) -> Result<Vec<Message>, DBError> {
    let limit = (limit as usize).min(100).max(1);

    // Get all messages for this channel
    let messages_map = self.messages.lock().await;
    let mut channel_messages: Vec<&Message> =
      messages_map.values().filter(|msg| msg.channel_id == channel_id).collect();

    // Sort by created_at (ULID timestamp) in descending order (newest first)
    channel_messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let result = match time {
      MessageTimePeriod::Relative { nearby } => {
        let side_limit = (limit / 2) + 1;

        // Find the index of the nearby message
        let nearby_index = channel_messages.iter().position(|msg| msg.id == nearby);

        match nearby_index {
          Some(idx) => {
            // Get older messages (before nearby)
            let older_start = if idx > side_limit { idx - side_limit } else { 0 };
            let older_messages: Vec<Message> =
              channel_messages[older_start..idx].iter().map(|&msg| msg.clone()).collect();

            // Get newer messages (after nearby, including nearby)
            let newer_end = (idx + side_limit + 1).min(channel_messages.len());
            let newer_messages: Vec<Message> =
              channel_messages[idx..newer_end].iter().map(|&msg| msg.clone()).collect();

            // Combine and remove duplicates
            let mut result = older_messages;
            result.extend(newer_messages);

            let mut seen = std::collections::HashSet::new();
            result.retain(|msg| seen.insert(msg.id.clone()));
            result
          }
          None => {
            // Nearby message not found, just return latest messages
            channel_messages.iter().take(limit as usize).map(|&msg| msg.clone()).collect()
          }
        }
      }
      MessageTimePeriod::Absolute { before, after, sort } => {
        let mut filtered: Vec<Message> = channel_messages
          .iter()
          .filter(|msg| {
            let mut include = true;
            if let Some(ref before_id) = before {
              if msg.id >= *before_id {
                include = false;
              }
            }
            if let Some(ref after_id) = after {
              if msg.id <= *after_id {
                include = false;
              }
            }
            include
          })
          .map(|&msg| msg.clone())
          .collect();

        // Apply sorting
        match sort.unwrap_or(MessageSort::Latest) {
          MessageSort::Latest => {
            // Already sorted descending by created_at
            filtered
          }
          MessageSort::Oldest => {
            filtered.reverse();
            filtered
          }
          _ => filtered,
        }
        .into_iter()
        .take(limit as usize)
        .collect()
      }
    };

    Ok(result)
  }
}
