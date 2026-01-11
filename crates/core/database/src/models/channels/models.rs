use std::collections::BTreeSet;

use chaty_proto::ChannelGroup;
use scylla::{DeserializeValue, SerializeValue};

use crate::models::files::FileDB;

#[derive(SerializeValue, DeserializeValue, Debug)]
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
  fn from(s: ChannelGroupDB) -> Self {
    Self {
      user_id: s.user_id,
      name: s.name,
      description: s.description,
      recipients: s.recipients.into_iter().collect(), // BTreeSet to Vec
      icon: s.icon.map(|f| chaty_proto::shared::v1::File {
        id: f.id,
        uploader_id: f.uploader_id,
        bucket: f.bucket,
        filename: f.filename,
        content_type: f.content_type,
        size: f.size,
        hash: f.hash,
        uploaded_at: f.uploaded_at.0,
        deleted: f.deleted,
        reported: f.reported,
      }),
      last_message_id: s.last_message_id,
      permissions: s.permissions,
      nsfw: s.nsfw,
    }
  }
}
