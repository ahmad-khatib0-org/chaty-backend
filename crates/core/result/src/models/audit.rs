use std::{borrow::Cow, collections::HashMap, fmt, sync::Arc};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::Context;

pub type AnyMap = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug, Clone, Display, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventName {
  UsersCreate,
  UsersLogin,
  UsersEmailConfirmation,
  UsersForgotPassword,
  UsersResetPassword,
  GroupsCreate,
  SearchUsernames,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventParameterKey {
  Data,
  UserId,
}

impl EventParameterKey {
  pub fn as_string(&self) -> Cow<'static, str> {
    match self {
      Self::UserId => Cow::Borrowed("user_id"),
      Self::Data => Cow::Borrowed("data"),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
  Fail,
  Success,
  Attempt,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, Default)]
#[display("AuditEventData: {parameters:?} {prior_state:?} {resulting_state:?}")]
pub struct AuditEventData {
  pub parameters: AnyMap,
  pub prior_state: AnyMap,
  pub resulting_state: AnyMap,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Display)]
#[display("EventStatus: {user_id} {session_id} {client} {ip_address} {x_forwarded_for}")]
pub struct AuditEventActor {
  pub user_id: String,
  pub session_id: String,
  pub client: String,
  pub ip_address: String,
  pub x_forwarded_for: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EventError {
  #[serde(skip_serializing_if = "String::is_empty")]
  pub description: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub status_code: Option<i32>,
}

impl fmt::Display for EventError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "EventError: {} {}", self.description, self.status_code.unwrap_or(0))
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Display)]
#[display("AuditRecord: {event_name} {status} {event} {actor} {meta:?}")]
pub struct AuditRecord {
  pub event_name: EventName,
  pub status: EventStatus,
  pub event: AuditEventData,
  pub actor: AuditEventActor,
  pub meta: AnyMap,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<EventError>,
}

impl AuditRecord {
  pub fn success(&mut self) {
    self.status = EventStatus::Success
  }

  pub fn fail(&mut self) {
    self.status = EventStatus::Fail;
  }

  pub fn new(ctx: Arc<Context>, event: EventName, initial_status: EventStatus) -> Self {
    Self {
      event_name: event,
      status: initial_status,
      actor: AuditEventActor {
        user_id: ctx.session.user_id.clone(),
        session_id: ctx.session.id.clone(),
        ip_address: ctx.ip_address.clone(),
        client: ctx.user_agent.clone(),
        x_forwarded_for: ctx.x_forwarded_for.clone(),
      },
      meta: HashMap::new(),
      event: AuditEventData {
        parameters: HashMap::new(),
        prior_state: HashMap::new(),
        resulting_state: HashMap::new(),
      },
      error: None,
    }
  }

  pub fn set_event_parameter(&mut self, key: EventParameterKey, val: Value) {
    self.event.parameters.insert(key.as_string().into_owned(), val);
  }

  pub fn set_prior_state(&mut self, data: AnyMap) {
    self.event.prior_state = data;
  }

  pub fn set_resulting_state(&mut self, data: AnyMap) {
    self.event.resulting_state = data;
  }
}
