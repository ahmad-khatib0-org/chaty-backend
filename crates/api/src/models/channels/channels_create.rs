use std::collections::HashMap;

use chaty_proto::{
  Channel, ChannelDirectMessage, ChannelGroup, ChannelSavedMessages, ChannelText,
  ChannelsCreateRequest, Server, User,
};
use chaty_utils::time::time_get_millis;
use ulid::Ulid;

pub fn channels_create_presave(
  user: &User,
  server: &Server,
  request: &ChannelsCreateRequest,
  direct: Option<ChannelDirectMessage>,
  group: Option<ChannelGroup>,
) -> Channel {
  let saved: Option<ChannelSavedMessages> = if request.channel_type == "saved".to_string() {
    Some(ChannelSavedMessages { user_id: user.id.clone() })
  } else {
    None
  };

  let text: Option<ChannelText> = if request.channel_type == "text".to_string() {
    Some(ChannelText {
      server_id: server.id.clone(),
      name: request.name.clone(),
      description: request.description.clone(),
      icon: None,
      last_message_id: None,
      default_permissions: None,
      role_permissions: HashMap::new(),
      nsfw: request.nsfw(),
    })
  } else {
    None
  };

  Channel {
    id: Ulid::new().to_string(),
    channel_type: request.channel_type.clone(),
    saved,
    direct,
    group,
    text,
    voice_max_users: Some(request.voice_max_users() as i32),
    created_at: time_get_millis(),
    updated_at: time_get_millis(),
  }
}
