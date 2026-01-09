use std::time::Duration;

use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::{Offset, TopicPartitionList};
use tokio::time::timeout;
use tracing::info;

use super::SearchWorkerController;

impl SearchWorkerController {
  /// Handle graceful shutdown for all consumers
  /// Waits for all in-flight tasks to complete before closing consumers
  pub async fn consumer_shutdown(&self) {
    info!("Beginning consumer shutdown - waiting for in-flight tasks...");

    let max_shutdown_wait = Duration::from_secs(60);
    let mut waited = Duration::ZERO;

    // Wait until all spawned tasks complete (with timeout)
    loop {
      let mut join_set = self.join_set.lock().await;
      if join_set.is_empty() {
        info!("All in-flight tasks completed. Closing all consumers...");
        drop(join_set);
        break;
      }

      let count = join_set.len();

      if waited >= max_shutdown_wait {
        info!(
          "Shutdown timeout reached (60s). {} tasks still running - force closing consumers",
          count
        );
        drop(join_set);
        break;
      }

      // Try to join the next task (with a timeout to avoid blocking forever)
      match timeout(Duration::from_millis(500), join_set.join_next()).await {
        Ok(Some(_)) => {
          // Task completed and was removed from join_set
          drop(join_set);
        }
        Ok(None) => {
          // join_set is empty
          info!("All in-flight tasks completed. Closing all consumers...");
          drop(join_set);
          break;
        }
        Err(_) => {
          // Timeout waiting for next task
          drop(join_set);
          info!("Waiting for {} in-flight tasks to complete", count);
          waited += Duration::from_millis(500);
        }
      }
    }

    // Final commit of any remaining tracked offsets before shutdown
    {
      info!("Acquiring locks for final commit phase...");
      let highest_offset = self.highest_offset.lock().await;
      info!("Got highest_offset lock. Offsets tracked: {}", highest_offset.len());

      if !highest_offset.is_empty() {
        info!("Flushing {} final offsets before shutdown...", highest_offset.len());
        let topic_to_consumer = self.topic_to_consumer.lock().await;
        info!("Got topic_to_consumer lock");
        let consumers_guard = self.consumers.lock().await;
        info!("Got consumers lock");

        for (topic, consumer_name) in topic_to_consumer.iter() {
          info!("Processing topic '{}' with consumer '{}'", topic, consumer_name);
          if let Some(consumer) = consumers_guard.get(consumer_name) {
            let mut tpl = TopicPartitionList::new();
            for ((t, partition), offset) in highest_offset.iter() {
              if t == topic {
                let commit_off = Offset::from_raw(*offset + 1);
                let _ = tpl.add_partition_offset(topic, *partition, commit_off);
              }
            }
            if tpl.count() > 0 {
              info!("Committing {} offsets to topic '{}'", tpl.count(), topic);
              match consumer.commit(&tpl, CommitMode::Sync) {
                Ok(_) => {
                  info!("Final commit of {} offsets for topic '{}' succeeded", tpl.count(), topic);
                }
                Err(err) => {
                  info!("Final commit for topic '{}' failed: {}", topic, err);
                }
              }
            }
          }
        }
      } else {
        info!("No offsets to commit");
      }
      info!("Final commit phase complete");
    }

    // Gracefully unsubscribe and close all consumers
    let consumers_guard = self.consumers.lock().await;
    for (consumer_name, consumer) in consumers_guard.iter() {
      consumer.unsubscribe();
      info!("Unsubscribed consumer '{}'", consumer_name);
    }
    drop(consumers_guard);

    info!("All consumers shutdown complete");
  }
}
