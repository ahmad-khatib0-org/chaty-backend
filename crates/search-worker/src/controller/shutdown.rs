use std::sync::atomic::Ordering;

use rdkafka::consumer::Consumer;
use tokio::{signal::ctrl_c, spawn};
use tracing::{error, info};

use super::SearchWorkerController;

impl SearchWorkerController {
  /// Start listening for shutdown signal (Ctrl+C)
  /// Coordinates graceful shutdown of message processing for all consumers
  pub fn shutdown_listener(&self) {
    let shutdown_notify = self.shutdown_notify.clone();
    let accepting = self.task_accepting.clone();
    let consumers = self.consumers.clone();
    let tx_metrics_shutdown = self.tx_metrics_shutdown.clone();

    spawn(async move {
      if let Err(err) = ctrl_c().await {
        error!("Error waiting for ctrl_c: {}", err);
        return;
      }

      info!("Shutdown signal received (Ctrl+C). Initiating graceful drain...");

      // Signal metrics server to shutdown
      let _ = tx_metrics_shutdown.send(());
      drop(tx_metrics_shutdown);

      // Stop accepting new tasks
      accepting.store(false, Ordering::SeqCst);
      info!("Stopped accepting new messages. Draining in-flight tasks...");

      // Pause all consumer partitions to stop further deliveries
      let consumers_guard = consumers.lock().await;
      for (consumer_name, consumer) in consumers_guard.iter() {
        match consumer.assignment() {
          Ok(tpl) => {
            if tpl.count() > 0 {
              if let Err(e) = consumer.pause(&tpl) {
                error!("Failed to pause consumer '{}' partitions: {}", consumer_name, e);
              } else {
                info!("Paused consumer '{}' partitions during shutdown.", consumer_name);
              }
            }
          }
          Err(err) => {
            error!("Could not get consumer '{}' assignment to pause: {}", consumer_name, err);
          }
        }
      }
      drop(consumers_guard);

      // Notify main loop to break
      shutdown_notify.notify_waiters();
    });
  }
}
