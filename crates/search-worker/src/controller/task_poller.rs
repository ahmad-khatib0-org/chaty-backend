use std::{
  io::{Error, ErrorKind},
  time::Duration,
};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use reqwest::Client;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::{
  models::tasks::{Task, TaskStatus},
  server::observability::MetricsCollector,
};

/// Poll a Meilisearch task until it completes (succeeds or fails)
pub async fn poll_task_until_complete(
  http: &Client,
  endpoint: &str,
  task_uid: &u64,
  api_key: &str,
  metrics: &MetricsCollector,
  index_name: &str,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    let path = "search-worker.controller.task_processor.poll_task_until_complete".into();
    let msg = msg.to_string();
    return InternalError { err_type: ErrorType::InternalError, temp: false, err, msg, path };
  };

  let url = format!("{}/tasks/{}", endpoint, task_uid);
  let poll_interval = Duration::from_millis(200);
  let max_wait = Duration::from_secs(15);  // Reduced from 30s for faster failure detection
  let mut waited = Duration::ZERO;
  debug!("Starting task poll for task_uid={}", task_uid);

  loop {
    if waited >= max_wait {
      let msg = "meilisearch task polling exceeded max wait time";
      return Err(Box::new(ie(Box::new(Error::new(ErrorKind::TimedOut, "task_timeout")), msg)));
    }

    sleep(poll_interval).await;
    waited += poll_interval;

    let mut req = http.get(&url);
    if !api_key.is_empty() {
      req = req.bearer_auth(api_key);
    }

    let res =
      req.send().await.map_err(|e| Box::new(ie(Box::new(e), "failed to poll task status")))?;

    if !res.status().is_success() {
      error!("Failed to poll task status: {}", res.status());
      continue;
    }

    let task: Task = res
       .json()
       .await
       .map_err(|err| Box::new(ie(Box::new(err), "failed to parse task response")))?;

     debug!("Task {} status: {:?}", task_uid, task.status);
     match task.status {
       TaskStatus::Succeeded => {
         debug!("Task {} succeeded", task_uid);
         return Ok(());
       }
      TaskStatus::Failed => {
        let error_msg =
          task.error.map(|e| e.message).unwrap_or_else(|| "unknown error".to_string());
        let msg = &format!("meilisearch task failed: {}", error_msg);
        return Err(Box::new(ie(Box::new(Error::new(ErrorKind::Other, "task_failed")), msg)));
      }
      TaskStatus::Canceled => {
        let msg = "meilisearch task was canceled";
        return Err(Box::new(ie(Box::new(Error::new(ErrorKind::Other, "task_canceled")), msg)));
      }
      TaskStatus::Enqueued | TaskStatus::Processing => {
        // Continue polling
        metrics.record_meili_retry(index_name);
      }
    }
  }
}
