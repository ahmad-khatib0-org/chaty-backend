use std::{
  collections::HashSet,
  io::{Error, ErrorKind},
  sync::Arc,
};

use async_trait::async_trait;
use chaty_proto::{Channel, GroupsListItem};
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{ChannelsRepository, ReferenceNoSqlDb};

#[async_trait]
impl ChannelsRepository for ReferenceNoSqlDb {
  async fn channels_groups_create(
    &self,
    ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    self.channels_insert(channel, ctx.session.user_id()).await
  }

  async fn channels_groups_list(
    &self,
    ctx: Arc<Context>,
    last_id: &str,
    limit: i32,
  ) -> Result<Vec<GroupsListItem>, DBError> {
    let channels = self.channels.lock().await;
    let user_id = ctx.session.user_id();

    let mut groups: Vec<GroupsListItem> = channels
      .values()
      .filter_map(|channel| {
        // Filter for group channels owned by the user
        if channel.channel_type == "group" {
          if let Some(group) = &channel.group {
            if group.user_id == user_id {
              return Some((channel.id.clone(), channel.clone()));
            }
          }
        }
        None
      })
      .collect::<Vec<_>>()
      .iter()
      .map(|(id, channel)| GroupsListItem {
        id: id.clone(),
        group: match &channel.group {
          Some(g) => Some(g.clone()),
          _ => None,
        },
        created_at: channel.created_at,
      })
      .collect();

    // Sort by ID descending
    groups.sort_by(|a, b| b.id.cmp(&a.id));

    // Apply cursor pagination
    if !last_id.is_empty() {
      if let Some(pos) = groups.iter().position(|g| g.id == last_id) {
        groups = groups[pos + 1..].to_vec();
      } else {
        groups.clear();
      }
    }

    // Apply limit
    groups.truncate(limit as usize);

    Ok(groups)
  }

  async fn channels_get_by_id(
    &self,
    _ctx: Arc<Context>,
    channel_id: &str,
  ) -> Result<Channel, DBError> {
    let channels = self.channels.lock().await;
    let path = "database.channels.channels_get_by_id".to_string();

    let chan = channels.iter().find(|chan| chan.1.id == channel_id);
    if chan.is_none() {
      let err = Box::new(Error::new(ErrorKind::NotFound, "channel not found"));
      return Err(DBError::new(path, err, ErrorType::NoRows, "channel is not found"));
    };

    Ok(chan.unwrap().1.clone())
  }

  async fn channels_get_channels_ids_by_user_id(
    &self,
    user_id: &str,
    channel_types: &[&str],
  ) -> Result<Vec<String>, DBError> {
    let channels = self.channels.lock().await;

    let type_set: HashSet<_> = channel_types.iter().cloned().collect();

    let channel_ids: Vec<String> = channels
      .iter()
      .filter(|(_id, channel)| {
        if !type_set.contains(channel.channel_type.as_str()) {
          return false;
        }

        // Check user participation based on channel data
        if &channel.channel_type == &"saved_messages".to_string() {
          return channel.saved.as_ref().unwrap().user_id == user_id;
        } else if &channel.channel_type == &"direct_message".to_string() {
          return channel.direct.as_ref().unwrap().recipients.contains(&user_id.to_string());
        } else if &channel.channel_type == &"group".to_string() {
          return channel.group.as_ref().unwrap().recipients.contains(&user_id.to_string());
        } else if &channel.channel_type == &"text".to_string() {
          return true;
        }
        false
      })
      .map(|(id, _)| id.clone())
      .collect();

    Ok(channel_ids)
  }

  async fn channels_insert(&self, channel: &Channel, _user_id: &str) -> Result<(), DBError> {
    let mut channels = self.channels.lock().await;
    let path = "database.channels.channels_insert".to_string();

    if channels.contains_key(&channel.id) {
      let msg = "channel already exists".to_string();
      Err(DBError { err_type: ErrorType::ResourceExists, msg, path, ..Default::default() })
    } else {
      channels.insert(channel.id.to_string(), channel.clone());
      Ok(())
    }
  }
}
