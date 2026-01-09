use std::{
  io::{Error, ErrorKind},
  time::Duration,
};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use reqwest::Client;
use serde_json::json;
use tokio::time::sleep;
use tracing::error;

use crate::{
  controller::usernames_task_processor::{delete_user_from_meili, push_user_to_meili},
  models::cdc::UserCDCMessage,
  server::observability::MetricsCollector,
};

/// Process a single user CDC message
pub async fn usernames_message_processor(
  payload: &str,
  http: &Client,
  endpoints: &[String],
  index_name: &str,
  api_key: &str,
  _dlq_topic: &str,
  metrics: &MetricsCollector,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    let path = "search-worker.controller.message_processor".into();
    let err_type = ErrorType::InternalError;
    return InternalError { err_type, temp: false, err, msg: msg.into(), path };
  };

  // Parse the CDC message
  let cdc_message: UserCDCMessage = serde_json::from_str(payload).map_err(|err| {
    error!("Raw CDC message payload: {}", payload);
    let e = err.to_string();
    Box::new(ie(Box::new(err), &format!("failed to deserialize user CDC message: {}", e)))
  })?;

  // Skip resolved markers (CockroachDB heartbeat messages)
  if cdc_message.resolved.is_some() {
    return Ok(());
  }

  let max_retries = 3;
  let mut backoff_ms = 100u64;

  // Determine operation type
  match (&cdc_message.after, &cdc_message.before) {
    // Create or Update: after exists
    (Some(after), _) => {
      let user_doc = json!({
        "id": after.id,
        "username": after.username,
        "display_name": after.display_name.clone().unwrap_or_default(),
        "profile_background_id": after.profile_background_id.clone().unwrap_or_default(),
      });

      let mut tries = 0;
      loop {
        tries += 1;
        match push_user_to_meili(&user_doc, http, endpoints, index_name, api_key, metrics).await {
          Ok(()) => return Ok(()),
          Err(err) => {
            if tries >= max_retries {
              error!("Failed to push user after {} retries: {}", max_retries, err);
              return Err(err);
            }
            error!("Failed to push user (try {}/{}): {}", tries, max_retries, err);
            metrics.record_meili_retry("users");
            sleep(Duration::from_millis(backoff_ms)).await;
            backoff_ms = (backoff_ms.saturating_mul(2)).min(5000);
          }
        }
      }
    }
    // Delete: after is None, before exists
    (None, Some(before)) => {
      let id = before.id.clone();

      let mut tries = 0;
      loop {
        tries += 1;
        match delete_user_from_meili(&id, http, endpoints, index_name, api_key, metrics).await {
          Ok(()) => return Ok(()),
          Err(err) => {
            if tries >= max_retries {
              error!("Failed to delete user after {} retries: {}", max_retries, err);
              return Err(err);
            }
            error!("Failed to delete user (try {}/{}): {}", tries, max_retries, err);
            metrics.record_meili_retry("users");
            sleep(Duration::from_millis(backoff_ms)).await;
            backoff_ms = (backoff_ms.saturating_mul(2)).min(5000);
          }
        }
      }
    }
    // Invalid: both None
    (None, None) => {
      let msg = "CDC message has neither after nor before state";
      let err = Box::new(Error::new(ErrorKind::InvalidData, "invalid_cdc_message"));
      return Err(Box::new(ie(err, msg)));
    }
  }
}
