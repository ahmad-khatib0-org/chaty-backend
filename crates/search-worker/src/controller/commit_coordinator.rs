use std::{collections::HashMap, mem::take, time::Duration};

use rdkafka::{
  consumer::{CommitMode, Consumer},
  Offset, TopicPartitionList,
};
use tokio::{spawn, time::interval};
use tracing::{debug, error};

use super::SearchWorkerController;

impl SearchWorkerController {
  /// Start periodic commit task for tracked offsets across all consumers
  pub fn periodic_commit(&self) {
    let highest = self.highest_offset.clone();
    let consumers = self.consumers.clone();
    let topic_to_consumer = self.topic_to_consumer.clone();
    let commit_interval_ms = 1000u64;

    spawn(async move {
      let mut ticker = interval(Duration::from_millis(commit_interval_ms));
      loop {
        ticker.tick().await;

        // Snapshot and clear the map
        let snapshot_map = {
          let mut guard = highest.lock().await;
          if guard.is_empty() {
            continue;
          }
          take(&mut *guard)
        };

        // Group offsets by topic so we can commit to the right consumer
        let mut offsets_by_topic: HashMap<String, Vec<((String, i32), i64)>> = HashMap::new();

        for ((topic, partition), offset) in snapshot_map.iter() {
          offsets_by_topic
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(((topic.clone(), *partition), *offset));
        }

        // Commit offsets for each topic using the appropriate consumer
        let topic_to_consumer_guard = topic_to_consumer.lock().await;
        let consumers_guard = consumers.lock().await;

        for (topic, offsets) in offsets_by_topic {
          // Look up which consumer is responsible for this topic
          match topic_to_consumer_guard.get(&topic) {
            Some(consumer_name) => {
              if let Some(consumer) = consumers_guard.get(consumer_name) {
                let mut tpl = TopicPartitionList::new();
                for ((_t, partition), offset) in offsets.iter() {
                  let commit_off = Offset::from_raw(*offset + 1);
                  let _ = tpl.add_partition_offset(&topic, *partition, commit_off);
                }

                if tpl.count() > 0 {
                  match consumer.commit(&tpl, CommitMode::Async) {
                    Ok(_) => {
                      debug!(
                        "Periodic batched commit dispatched for {} offsets from topic {} using consumer '{}'",
                        tpl.count(),
                        topic,
                        consumer_name
                      );
                    }
                    Err(err) => {
                      error!(
                        "Periodic commit error for topic {} on consumer '{}': {} — will retry",
                        topic, consumer_name, err
                      );
                      // Re-merge the snapshot back into highest map, keeping max offsets
                      let mut guard = highest.lock().await;
                      for ((t, p), offset) in offsets.iter() {
                        let prev = guard.get(&(t.clone(), *p)).copied().unwrap_or(-1);
                        if *offset > prev {
                          guard.insert((t.clone(), *p), *offset);
                        }
                      }
                    }
                  }
                }
              } else {
                error!(
                  "Consumer '{}' for topic '{}' not found in consumers map",
                  consumer_name, topic
                );
                // Re-merge offsets back
                let mut guard = highest.lock().await;
                for ((t, p), offset) in offsets.iter() {
                  let prev = guard.get(&(t.clone(), *p)).copied().unwrap_or(-1);
                  if *offset > prev {
                    guard.insert((t.clone(), *p), *offset);
                  }
                }
              }
            }
            None => {
              error!("No consumer mapping found for topic '{}'. Dropping offsets.", topic);
            }
          }
        }
        drop(topic_to_consumer_guard);
        drop(consumers_guard);
      }
    });
  }
}
