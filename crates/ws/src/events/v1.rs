use chaty_proto::{
  Channel, ChannelVoiceState, Emoji, Message, MessageWebhook, Server, ServerMember, User,
};
use chaty_result::errors::ErrorType;
use deadpool_redis::{redis::AsyncCommands, Connection};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Ping Packet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Ping {
  Binary(Vec<u8>),
  Number(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventV1 {
  /// Multiple events
  Bulk {
    v: Vec<EventV1>,
  },
  /// Error event
  Error {
    typ: ErrorType,
  },

  /// Successfully authenticated
  Authenticated,
  /// Logged out
  Logout,
  /// Basic data to cache
  Ready {},

  /// Ping response
  Pong {
    data: Ping,
  },
  /// New message
  Message(Message),

  /// Update existing message
  MessageUpdate {},

  /// Append information to existing message
  MessageAppend {},

  /// Delete message
  MessageDelete {
    id: String,
    channel: String,
  },

  /// New reaction to a message
  MessageReact {
    id: String,
    channel_id: String,
    user_id: String,
    emoji_id: String,
  },

  /// Remove user's reaction from message
  MessageUnreact {
    id: String,
    channel_id: String,
    user_id: String,
    emoji_id: String,
  },

  /// Remove a reaction from message
  MessageRemoveReaction {
    id: String,
    channel_id: String,
    emoji_id: String,
  },

  /// Bulk delete messages
  BulkMessageDelete {
    channel: String,
    ids: Vec<String>,
  },

  /// New server
  ServerCreate {
    id: String,
    server: Server,
    channels: Vec<Channel>,
    emojis: Vec<Emoji>,
    voice_states: Vec<ChannelVoiceState>,
  },

  /// Update existing server
  ServerUpdate {},

  /// Delete server
  ServerDelete {
    id: String,
  },

  /// Update existing server member
  ServerMemberUpdate {},

  /// User joins server
  ServerMemberJoin {
    id: String,
    member: ServerMember,
  },

  /// User left server
  ServerMemberLeave {},

  /// Server role created or updated
  ServerRoleUpdate {},

  /// Server role deleted
  ServerRoleDelete {
    id: String,
    role_id: String,
  },

  /// Server roles ranks updated
  ServerRoleRanksUpdate {
    id: String,
    ranks: Vec<String>,
  },

  /// Update existing user
  UserUpdate {},

  /// Relationship with another user changed
  UserRelationship {
    id: String,
    user: User,
  },
  /// Settings updated remotely
  UserSettingsUpdate {},

  /// User has been platform banned or deleted their account
  ///
  /// Clients should remove the following associated data:
  /// - Messages
  /// - DM Channels
  /// - Relationships
  /// - Server Memberships
  ///
  /// User flags are specified to explain why a wipe is occurring though not all reasons will necessarily ever appear.
  UserPlatformWipe {
    user_id: String,
    flags: i32,
  },

  /// New emoji
  EmojiCreate(Emoji),

  /// Delete emoji
  EmojiDelete {
    id: String,
  },

  /// New report
  ReportCreate(),
  /// New channel
  ChannelCreate(Channel),

  /// Update existing channel
  ChannelUpdate {},

  /// Delete channel
  ChannelDelete {
    id: String,
  },

  /// User joins a group
  ChannelGroupJoin {
    id: String,
    user: String,
  },

  /// User leaves a group
  ChannelGroupLeave {
    id: String,
    user: String,
  },

  /// User started typing in a channel
  ChannelStartTyping {
    id: String,
    user: String,
  },

  /// User stopped typing in a channel
  ChannelStopTyping {
    id: String,
    user: String,
  },

  /// User acknowledged message in channel
  ChannelAck {
    id: String,
    user: String,
    message_id: String,
  },

  /// New webhook
  WebhookCreate(MessageWebhook),

  /// Update existing webhook
  WebhookUpdate {},

  /// Delete webhook
  WebhookDelete {
    id: String,
  },

  /// Auth events
  Auth(),

  /// Voice events
  VoiceChannelJoin {},
  VoiceChannelLeave {
    id: String,
    user: String,
  },
  VoiceChannelMove {},
  UserVoiceStateUpdate {},
  UserMoveVoiceChannel {
    node: String,
    from: String,
    to: String,
    token: String,
  },
}

impl EventV1 {
  pub async fn p(self, conn: &mut Connection, channel: String) {
    #[cfg(debug_assertions)]
    info!("Publishing event to {channel}: {self:?}");

    // Serialize to JSON string
    let json = match serde_json::to_string(&self) {
      Ok(s) => s,
      Err(e) => {
        error!("Failed to serialize EventV1: {}", e);
        return;
      }
    };

    info!("Publishing event to {channel}: {json}");

    let _: () = conn.publish(&channel, json).await.unwrap_or_else(|e| {
      error!("Failed to publish: {}", e);
    });
  }

  /// Publish private event
  pub async fn private(self, conn: &mut Connection, id: String) {
    self.p(conn, format!("{id}!")).await;
  }
}
