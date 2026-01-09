use std::{
  collections::HashMap,
  str::{from_utf8, Utf8Error},
  sync::atomic::Ordering,
  time::{Duration, Instant},
};

use base64::engine::{general_purpose, Engine as _};
use rdkafka::{
  consumer::{CommitMode, Consumer},
  producer::{FutureProducer, FutureRecord},
  Message,
};
use serde_json::Value;
use tokio::{select, sync::Mutex, time::sleep};
use tracing::{debug, error, info};

use crate::controller::usernames_message_processor::usernames_message_processor;

use super::SearchWorkerController;

impl SearchWorkerController {
  /// Send a message to the DLQ when UTF-8 parsing fails
  async fn send_utf8_error_to_dlq(
    producer: &FutureProducer,
    dlq_topic: &str,
    payload_bytes: &[u8],
    utf8_err: Utf8Error,
  ) {
    let dlq_obj = serde_json::json!({
      "original_bytes_base64": general_purpose::STANDARD.encode(payload_bytes),
      "error": format!("invalid utf8: {}", utf8_err),
      "ts": chrono::Utc::now().timestamp_millis()
    });
    let _ = producer
      .send(
        FutureRecord::to(dlq_topic).payload(&dlq_obj.to_string()).key(""),
        Duration::from_secs(1),
      )
      .await;
  }

  /// Send a message to the DLQ when processing fails
  async fn send_processing_error_to_dlq(
    producer: &FutureProducer,
    dlq_topic: &str,
    payload_str: &str,
    error_msg: &str,
  ) {
    let original_json =
      serde_json::from_str::<Value>(payload_str).unwrap_or(Value::String(payload_str.to_string()));
    let dlq_obj = serde_json::json!({
      "original": original_json,
      "error": error_msg,
      "ts": chrono::Utc::now().timestamp_millis()
    });
    let _ = producer
      .send(
        FutureRecord::to(dlq_topic).payload(&dlq_obj.to_string()).key(""),
        Duration::from_secs(1),
      )
      .await;
  }

  /// Record an offset as processed in the highest_offset map
  async fn mark_offset_processed(
    highest_offset: &Mutex<HashMap<(String, i32), i64>>,
    topic: String,
    partition: i32,
    offset: i64,
  ) {
    let mut guard = highest_offset.lock().await;
    let key = (topic.clone(), partition);
    let prev = guard.get(&key).copied().unwrap_or(-1);
    if offset > prev {
      guard.insert(key, offset);
    }
    debug!("Marked processed offset {} for {}[{}]", offset, topic, partition);
  }

  /// Consumer for user CDC changes
  /// Processes messages from the search.users.changes topic
  pub async fn usernames_consumer(&self) {
    let config = self.config.clone();
    let http = self.http_client.clone();
    let metrics = self.metrics.clone();
    let highest_offset = self.highest_offset.clone();
    let semaphore = self.semaphore.clone();
    let join_set = self.join_set.clone();
    let consumers = self.consumers.clone();
    let producer = self.producer.clone();
    let task_accepting = self.task_accepting.clone();
    let shutdown_notify = self.shutdown_notify.clone();

    let index_name = config.search.index_usernames.clone();
    let dlq_topic = config.topics.search_users_changes_dlq.clone();
    let api_key = config.search.api_key.clone();
    let endpoints = if !config.search.endpoints.is_empty() {
      config.search.endpoints.clone()
    } else {
      vec![config.search.host.clone()]
    };

    // Get the usernames consumer from the consumers map
    let consumer = {
      let consumers_guard = consumers.lock().await;
      consumers_guard.get("usernames").cloned()
    };

    let consumer = match consumer {
      Some(c) => c,
      None => {
        error!("Usernames consumer not found in controllers map");
        return;
      }
    };

    loop {
      select! {
        _ = shutdown_notify.notified() => {
          info!("Shutdown requested — breaking consumption loop.");
          break;
        }
        maybe_msg = consumer.recv() => {
          match maybe_msg {
            Err(e) => {
              error!("Kafka receive error: {}", e);
              sleep(Duration::from_secs(1)).await;
            }
            Ok(msg) => {
              // Extract and validate payload
              let payload_str = if let Some(payload_bytes) = msg.payload() {
                match from_utf8(payload_bytes) {
                  Ok(s) => Some(s.to_string()),
                  Err(utf8_err) => {
                    error!("Invalid UTF-8 in message: {}", utf8_err);
                    Self::send_utf8_error_to_dlq(&producer, &dlq_topic, payload_bytes, utf8_err).await;
                    if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                      error!("Failed to commit offset for invalid-utf8 message: {}", e);
                    }
                    metrics.record_message_failed("users");
                    continue;
                  }
                }
              } else {
                // Empty payload - skip and commit
                if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                  error!("Failed to commit offset for empty payload: {}", e);
                }
                continue;
              };

              let payload_str = payload_str.unwrap();

              // Extract message metadata
              let key_topic = msg.topic().to_string();
              let key_partition = msg.partition();
              let key_offset = msg.offset();

              if task_accepting.load(Ordering::SeqCst) {
                let semaphore_permit = semaphore.clone().acquire_owned();
                let highest = highest_offset.clone();
                let join = join_set.clone();
                let http_clone = http.clone();
                let metrics_clone = metrics.clone();
                let endpoints_clone = endpoints.clone();
                let index_clone = index_name.clone();
                let api_key_clone = api_key.clone();
                let dlq_clone = dlq_topic.clone();
                let prod = producer.clone();

                // Spawn task for async processing
                join.lock().await.spawn(async move {
                  let _permit = match semaphore_permit.await {
                    Ok(p) => p,
                    Err(_) => {
                      error!("Semaphore closed unexpectedly");
                      return;
                    }
                  };

                  let start = Instant::now();

                  let result = usernames_message_processor(
                    &payload_str,
                    &http_clone,
                    &endpoints_clone,
                    &index_clone,
                    &api_key_clone,
                    &dlq_clone,
                    &metrics_clone,
                  )
                  .await;

                  match result {
                    Ok(()) => {
                      Self::mark_offset_processed(&highest, key_topic.clone(), key_partition, key_offset).await;
                      metrics_clone.record_message_processed();
                    }
                    Err(err) => {
                      error!(
                        "Processing failed for message {}[{}] @ {}: {}",
                        key_topic, key_partition, key_offset, err
                      );
                      Self::send_processing_error_to_dlq(&prod, &dlq_clone, &payload_str, &format!("{}", err)).await;
                      Self::mark_offset_processed(&highest, key_topic.clone(), key_partition, key_offset).await;
                      metrics_clone.record_message_failed("users");
                    }
                  }

                  let elapsed = start.elapsed();
                  metrics_clone.observe_meili_indexing_duration("users", elapsed.as_secs_f64());
                });
              } else {
                // Draining mode - process inline
                info!("Draining mode: processing message inline before shutdown.");
                let start = Instant::now();

                let result = usernames_message_processor(
                  &payload_str,
                  &http,
                  &endpoints,
                  &index_name,
                  &api_key,
                  &dlq_topic,
                  &metrics,
                )
                .await;

                match result {
                   Ok(()) => {
                     if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                       error!("Failed to commit offset during drain: {}", e);
                     }
                     metrics.record_message_processed();
                   }
                   Err(err) => {
                     error!("Inline processing failed during drain: {}", err);
                     Self::send_processing_error_to_dlq(&producer, &dlq_topic, &payload_str, &format!("{}", err)).await;
                     if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                       error!("Failed to commit offset after DLQ during drain: {}", e);
                     }
                     metrics.record_message_failed("users");
                   }
                 }

                let elapsed = start.elapsed();
                metrics.observe_meili_indexing_duration("users", elapsed.as_secs_f64());
              }
            }
          }
        }
      }
    }
  }
}
