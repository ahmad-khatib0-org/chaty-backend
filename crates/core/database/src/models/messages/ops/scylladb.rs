use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{Message, MessageSort};
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType},
};
use futures::try_join;

use crate::{
  models::messages::{MessageTimePeriod, MessagesRepository},
  ScyllaDb,
};

#[async_trait]
impl MessagesRepository for ScyllaDb {
  async fn messages_get_by_channel_id(
    &self,
    _ctx: Arc<Context>,
    channel_id: String,
    limit: i32,
    time: MessageTimePeriod,
  ) -> Result<Vec<Message>, DBError> {
    let path = "database.messages.messages_get_by_channel_id".to_string();

    let de = |err: BoxedErr, msg: &str| {
      let err_type = ErrorType::DBSelectError;
      DBError { path: path.clone(), err_type, msg: msg.to_string(), err }
    };

    let limit = (limit as i64).min(100).max(1);

    let messages = match time {
      MessageTimePeriod::Relative { nearby } => {
        let side_limit = (limit / 2) + 1;

        // Execute both queries concurrently using try_join!
        let (newer_messages, older_messages): (Vec<Message>, Vec<Message>) = try_join!(
          async {
            let rows = self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id_gte,
                (channel_id.clone(), nearby.clone(), side_limit),
              )
              .await
              .map_err(|e| de(Box::new(e), "failed to fetch newer messages"))?
              .into_rows_result()
              .map_err(|e| de(Box::new(e), "failed to parse newer rows"))?;

            let messages: Vec<Message> = rows
              .rows::<Message>()
              .map_err(|e| de(Box::new(e), "failed to iterate over newer rows"))?
              .filter_map(|row| row.ok())
              .collect();

            Ok::<_, DBError>(messages)
          },
          async {
            let rows = self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id_lt,
                (channel_id.clone(), nearby.clone(), side_limit),
              )
              .await
              .map_err(|e| de(Box::new(e), "failed to fetch older messages"))?
              .into_rows_result()
              .map_err(|e| de(Box::new(e), "failed to parse older rows"))?;

            let messages: Vec<Message> = rows
              .rows::<Message>()
              .map_err(|e| de(Box::new(e), "failed to iterate over older rows"))?
              .filter_map(|row| row.ok())
              .collect();

            Ok::<_, DBError>(messages)
          }
        )?;

        // Combine and remove duplicates
        let mut result = older_messages;
        result.extend(newer_messages);

        let mut seen = std::collections::HashSet::new();
        result.retain(|msg| seen.insert(msg.id.clone()));
        result
      }
      MessageTimePeriod::Absolute { before, after, sort } => {
        let rows_result = match (before, after, sort) {
          (Some(before), Some(after), _) => {
            self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id_range,
                (channel_id.clone(), after, before, limit),
              )
              .await
          }
          (Some(before), None, _) => {
            self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id_lt,
                (channel_id.clone(), before, limit),
              )
              .await
          }
          (None, Some(after), _) => {
            self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id_gt,
                (channel_id.clone(), after, limit),
              )
              .await
          }
          (None, None, _) => {
            self
              .db
              .execute_unpaged(
                &self.prepared.messages.get_messages_by_channel_id,
                (channel_id.clone(), limit),
              )
              .await
          }
        };

        let rows = rows_result
          .map_err(|e| de(Box::new(e), "failed to fetch messages"))?
          .into_rows_result()
          .map_err(|e| de(Box::new(e), "failed to parse rows"))?;

        let mut messages: Vec<Message> = rows
          .rows::<Message>()
          .map_err(|e| de(Box::new(e), "failed to iterate over rows"))?
          .filter_map(|row| row.ok())
          .collect();

        // Apply sorting
        match sort.unwrap_or(MessageSort::Latest) {
          MessageSort::Latest => messages,
          MessageSort::Oldest => {
            messages.reverse();
            messages
          }
          _ => messages,
        }
      }
    };

    Ok(messages.into_iter().take(limit as usize).collect())
  }
}
