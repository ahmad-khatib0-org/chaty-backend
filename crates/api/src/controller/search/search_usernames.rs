use std::{io::ErrorKind, sync::Arc};

use chaty_proto::{
  search_usernames_response::Response::{Data, Error},
  SearchUser, SearchUsernamesRequest, SearchUsernamesResponse, SearchUsernamesResponseData,
};
use chaty_result::{
  audit::{AuditRecord, EventName, EventParameterKey, EventStatus},
  context::Context,
  errors::{AppError, AppErrorErrors, BoxedErr, ERROR_ID_INTERNAL},
};
use serde_json::json;
use tokio::{spawn, sync::Mutex};
use tonic::{Code, Request, Response, Status};

use crate::controller::{audit::process_audit, ApiController};

pub async fn search_usernames(
  ctr: &ApiController,
  request: Request<SearchUsernamesRequest>,
) -> Result<Response<SearchUsernamesResponse>, Status> {
  let start = std::time::Instant::now();
  let ctx = request.extensions().get::<Arc<Context>>().cloned().unwrap();
  let path = "api.search.search_usernames";
  let req = request.into_inner();

  let mut audit = AuditRecord::new(ctx.clone(), EventName::SearchUsernames, EventStatus::Fail);

  let req_clone = req.clone();
  let audit_future = spawn(async move { search_usernames_auditable(&req_clone) });
  let audit_slot = Arc::new(Mutex::new(Some(audit_future)));

  let get_audit = || async {
    let mut slot = audit_slot.lock().await;
    let handle = slot.take().expect("audit handle already taken");
    handle.await.unwrap_or_else(|e| json!({ "error": format!("{e}") }))
  };

  let mut audit_clone = audit.clone();
  let return_err = move |e: AppError| async move {
    let data = get_audit().await;
    audit_clone.set_event_parameter(EventParameterKey::Data, data);
    process_audit(&audit_clone);
    Response::new(SearchUsernamesResponse { response: Some(Error(e.to_proto())) })
  };

  let ie = |err: BoxedErr| {
    let errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    AppError::new(ctx.clone(), path, ERROR_ID_INTERNAL, None, "", Code::Internal.into(), errors)
  };

  if req.query.trim().is_empty() {
    ctr.metrics.record_search_usernames_failure();
    let id = "search.empty_query";
    let err = AppError::new(ctx.clone(), path, id, None, "", Code::InvalidArgument.into(), None);
    return Ok(return_err(err).await);
  }

  // Limit results to 5 if not specified or if greater than 5
  let limit = if req.limit <= 0 || req.limit > 5 { 5 } else { req.limit as usize };

  let index_name = ctr.config.search.index_usernames.clone();
  let api_key = ctr.config.search.api_key.clone();
  
  // Use endpoints vector if available, otherwise fall back to host
  let endpoint = if !ctr.config.search.endpoints.is_empty() {
    ctr.config.search.endpoints[0].clone()
  } else {
    ctr.config.search.host.clone()
  };

  let db_start = std::time::Instant::now();
  ctr.metrics.record_db_operation("search_usernames");

  let search_url = format!("{}/indexes/{}/search", endpoint, index_name);

  let search_payload = json!({ "q": req.query, "limit": limit });

  let result = ctr
    .http_client
    .post(&search_url)
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&search_payload)
    .send()
    .await;

  let db_duration = db_start.elapsed().as_secs_f64();
  ctr.metrics.observe_db_operation_duration("search_usernames", db_duration);

  let response_data = match result {
    Ok(response) => match response.json::<serde_json::Value>().await {
      Ok(data) => {
        if let Some(hits) = data.get("hits").and_then(|h| h.as_array()) {
          let users = hits
            .iter()
            .filter_map(|hit| {
              let id = hit.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
              let username = hit.get("username").and_then(|v| v.as_str()).map(|s| s.to_string());
              let display_name = hit
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
              let avatar = hit
                .get("avatar")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

              match (id, username) {
                (Some(id), Some(username)) => {
                  Some(SearchUser { id, username, display_name, avatar })
                }
                _ => None,
              }
            })
            .collect();

          SearchUsernamesResponseData { users }
        } else {
          ctr.metrics.record_db_error("search_usernames", "invalid_response_format");
          ctr.metrics.record_search_usernames_failure();
          let err = ie(Box::new(std::io::Error::new(
            ErrorKind::InvalidData,
            "Invalid Meilisearch response format",
          )));
          return Ok(return_err(err).await);
        }
      }
      Err(err) => {
        tracing::error!("Failed to parse Meilisearch response: {:?}", err);
        ctr.metrics.record_db_error("search_usernames", &err.to_string());
        ctr.metrics.record_search_usernames_failure();
        return Ok(return_err(ie(Box::new(err))).await);
      }
    },
    Err(err) => {
      tracing::error!("Failed to search usernames: {:?}", err);
      ctr.metrics.record_db_error("search_usernames", &err.to_string());
      ctr.metrics.record_search_usernames_failure();
      return Ok(return_err(ie(Box::new(err))).await);
    }
  };

  audit.set_event_parameter(EventParameterKey::Data, get_audit().await);
  audit.success();
  process_audit(&audit);

  let request_duration = start.elapsed().as_secs_f64();
  ctr.metrics.observe_request_duration("search.search_usernames", request_duration);
  ctr.metrics.record_search_usernames_success();

  Ok(Response::new(SearchUsernamesResponse { response: Some(Data(response_data)) }))
}

fn search_usernames_auditable(req: &SearchUsernamesRequest) -> serde_json::Value {
  json!({ "query": req.query, "limit": req.limit })
}
