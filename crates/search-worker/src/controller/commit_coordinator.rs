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
        let consumers_guard = consumers.lock().await;
        for (topic, offsets) in offsets_by_topic {
          // Try to find a consumer that is subscribed to this topic
          // For simplicity, we'll commit to any consumer that has this topic
          for (_consumer_name, consumer) in consumers_guard.iter() {
            let mut tpl = TopicPartitionList::new();
            for ((_t, partition), offset) in offsets.iter() {
              let commit_off = Offset::from_raw(*offset + 1);
              let _ = tpl.add_partition_offset(&topic, *partition, commit_off);
            }

            if tpl.count() > 0 {
              match consumer.commit(&tpl, CommitMode::Async) {
                Ok(_) => {
                  debug!(
                    "Periodic batched commit dispatched for {} to topic {}",
                    tpl.count(),
                    topic
                  );
                  break; // Committed successfully, move to next topic
                }
                Err(err) => {
                  error!("Periodic commit error for topic {}: {} — will retry", topic, err);
                  // Re-merge the snapshot back into highest map, keeping max offsets
                  let mut guard = highest.lock().await;
                  for ((t, p), offset) in offsets.iter() {
                    let prev = guard.get(&(t.clone(), *p)).copied().unwrap_or(-1);
                    if *offset > prev {
                      guard.insert((t.clone(), *p), *offset);
                    }
                  }
                  break; // Move to next topic after error
                }
              }
            }
          }
        }
        drop(consumers_guard);
      }
    });
  }
}
