use std::time::Duration;

use rdkafka::consumer::Consumer;
use tokio::time::sleep;
use tracing::info;

use super::SearchWorkerController;

impl SearchWorkerController {
  /// Handle graceful shutdown for all consumers
  /// Waits for all in-flight tasks to complete before closing consumers
  pub async fn consumer_shutdown(&self) {
    info!("Beginning consumer shutdown - waiting for in-flight tasks...");

    // Wait until all spawned tasks complete
    loop {
      let join_set = self.join_set.lock().await;
      if join_set.is_empty() {
        info!("All in-flight tasks completed. Closing all consumers...");
        break;
      }

      let count = join_set.len();
      drop(join_set);

      info!("Waiting for {} in-flight tasks to complete", count);
      sleep(Duration::from_millis(500)).await;
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
