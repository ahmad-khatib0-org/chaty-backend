use std::collections::{BTreeMap, BTreeSet};

use chaty_proto::{
  ChannelDirectMessage, ChannelGroup, ChannelSavedMessages, ChannelText, OverrideField,
};
use scylla::{value::CqlTimestamp, DeserializeValue, SerializeValue};

use crate::models::files::FileDB;

#[derive(Debug, Clone)]
pub struct ChannelDB {
  pub id: String,
  pub channel_type: String,
  pub saved: Option<ChannelSavedMessages>,
  pub direct: Option<ChannelDirectMessage>,
  pub group: Option<ChannelGroupDB>,
  pub text: Option<ChannelSavedMessages>,
  pub created_at: CqlTimestamp,
  pub updated_at: CqlTimestamp,
}

#[derive(SerializeValue, DeserializeValue, Debug, Clone)]
pub struct ChannelGroupDB {
  pub user_id: String,
  pub name: String,
  pub description: Option<String>,
  pub recipients: BTreeSet<String>,
  pub icon: Option<FileDB>,
  pub last_message_id: Option<String>,
  pub permissions: Option<i64>,
  pub nsfw: bool,
}

impl From<ChannelGroupDB> for ChannelGroup {
  fn from(ch: ChannelGroupDB) -> Self {
    Self {
      user_id: ch.user_id,
      name: ch.name,
      description: ch.description,
      recipients: ch.recipients.into_iter().collect(), // BTreeSet to Vec
      icon: ch.icon.map(|f| f.into()),
      last_message_id: ch.last_message_id,
      permissions: ch.permissions,
      nsfw: ch.nsfw,
    }
  }
}

#[derive(SerializeValue, DeserializeValue, Debug)]
pub struct ChannelTextDB {
  pub server_id: String,
  pub name: String,
  pub description: Option<String>,
  pub icon: Option<FileDB>,
  pub last_message_id: Option<String>,
  pub default_permissions: Option<OverrideField>,
  pub role_permissions: BTreeMap<String, OverrideField>,
  pub nsfw: bool,
}

impl From<ChannelTextDB> for ChannelText {
  fn from(ch: ChannelTextDB) -> Self {
    Self {
      server_id: ch.server_id,
      name: ch.name,
      description: ch.description,
      icon: ch.icon.map(|f| f.into()),
      last_message_id: ch.last_message_id,
      default_permissions: ch.default_permissions.map(|p| p.into()),
      role_permissions: ch.role_permissions.into_iter().map(|(k, v)| (k, v.into())).collect(),
      nsfw: ch.nsfw,
    }
  }
}
