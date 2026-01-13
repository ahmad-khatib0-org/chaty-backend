use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chaty_proto::{channel::ChannelData, Channel, GroupsListItem};
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{ChannelsRepository, ReferenceNoSqlDb};

#[async_trait]
impl ChannelsRepository for ReferenceNoSqlDb {
  async fn channels_groups_create(
    &self,
    _ctx: Arc<Context>,
    channel: &Channel,
  ) -> Result<(), DBError> {
    let mut channels = self.channels.lock().await;
    let path = "database.channels.channels_create".to_string();

    if channels.contains_key(&channel.id) {
      let msg = "channel already exists".to_string();
      Err(DBError { err_type: ErrorType::ResourceExists, msg, path, ..Default::default() })
    } else {
      channels.insert(channel.id.to_string(), channel.clone());
      Ok(())
    }
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
          if let Some(ChannelData::Group(group)) = &channel.channel_data {
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
        group: match &channel.channel_data {
          Some(ChannelData::Group(g)) => Some(g.clone()),
          _ => None,
        },
        created_at: channel.created_at.as_ref().map(|ts| ts.seconds).unwrap_or(0),
      })
      .collect();

    // Sort by ID descending (ULID order = reverse chronological)
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
        match &channel.channel_data {
          Some(ChannelData::Direct(dm)) => dm.recipients.contains(&user_id.to_string()),
          Some(ChannelData::Group(group)) => group.recipients.contains(&user_id.to_string()),
          Some(ChannelData::Saved(saved)) => saved.user_id == user_id,
          Some(ChannelData::Text(_)) => true,
          None => false,
        }
      })
      .map(|(id, _)| id.clone())
      .collect();

    Ok(channel_ids)
  }
}
