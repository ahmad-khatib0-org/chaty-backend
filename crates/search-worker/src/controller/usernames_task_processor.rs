use std::io::{Error, ErrorKind};

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use reqwest::Client;

use crate::{
  controller::task_poller::poll_task_until_complete, models::tasks::TaskResponse,
  server::observability::MetricsCollector,
};

/// Push user document to Meilisearch and wait for task completion
pub async fn push_user_to_meili(
  user_doc: &serde_json::Value,
  http: &Client,
  endpoints: &[String],
  index_name: &str,
  api_key: &str,
  metrics: &MetricsCollector,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    let path = "search-worker.controller.task_processor.push_user_to_meili".into();
    let err_type = ErrorType::InternalError;
    return InternalError { err_type, temp: false, err, msg: msg.into(), path };
  };

  let url = format!("{}/indexes/{}/documents", &endpoints[0], index_name);

  let mut req = http.post(&url).json(user_doc);
  if !api_key.is_empty() {
    req = req.bearer_auth(api_key);
  }

  let resp = req
    .send()
    .await
    .map_err(|e| Box::new(ie(Box::new(e), "failed to post document to meilisearch")))?;

  let status = resp.status();
  if !status.is_success() {
    let txt = resp.text().await.unwrap_or_default();
    let err = Box::new(Error::new(ErrorKind::Other, "http_response_error"));
    let msg = &format!("meilisearch returned error: status={}, body={}", status, txt);
    return Err(Box::new(ie(err, msg)));
  }

  let response: TaskResponse = resp
    .json()
    .await
    .map_err(|err| Box::new(ie(Box::new(err), "failed to parse meilisearch response")))?;

  poll_task_until_complete(http, &endpoints[0], &response.task_uid, api_key, metrics, index_name)
    .await
}

/// Delete user document from Meilisearch and wait for task completion
pub async fn delete_user_from_meili(
  user_id: &str,
  http: &Client,
  endpoints: &[String],
  index_name: &str,
  api_key: &str,
  metrics: &MetricsCollector,
) -> Result<(), BoxedErr> {
  let ie = |err: BoxedErr, msg: &str| {
    let path = "search-worker.controller.task_processor.delete_user_from_meili".into();
    let err_type = ErrorType::InternalError;
    return InternalError { err_type, temp: false, err, msg: msg.into(), path };
  };

  let url = format!("{}/indexes/{}/documents/{}", &endpoints[0], index_name, user_id);

  let mut req = http.delete(&url);
  if !api_key.is_empty() {
    req = req.bearer_auth(api_key);
  }

  let resp = req.send().await.map_err(|e| {
    Box::new(ie(Box::new(e), "failed to delete document from meilisearch")) as BoxedErr
  })?;

  let status = resp.status();
  if !status.is_success() {
    let txt = resp.text().await.unwrap_or_default();
    let err = Box::new(Error::new(ErrorKind::Other, "http_response_error"));
    let msg = &format!("meilisearch returned error: status={}, body={}", status, txt);
    return Err(Box::new(ie(err, msg)));
  }

  let response: TaskResponse = resp.json().await.map_err(|err| {
    Box::new(ie(Box::new(err), "failed to parse meilisearch response")) as BoxedErr
  })?;

  poll_task_until_complete(http, &endpoints[0], &response.task_uid, api_key, metrics, index_name)
    .await
}
