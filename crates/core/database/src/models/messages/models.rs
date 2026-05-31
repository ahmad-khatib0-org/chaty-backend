use chaty_proto::{MessageSort, MessageSystem};

/// Message Query
pub struct MessageQuery {
  /// Maximum number of messages to fetch
  ///
  /// For fetching nearby messages, this is \`(limit + 1)\`.
  pub limit: Option<i64>,
  /// Filter to apply
  pub filter: MessageFilter,
  /// Time period to fetch
  pub time_period: MessageTimePeriod,
}

/// Message Filter
#[derive(Default)]
pub struct MessageFilter {
  /// Parent channel ID
  pub channel: Option<String>,
  /// Message author ID
  pub author: Option<String>,
  /// Search query
  pub query: Option<String>,
  /// Search for pinned
  pub pinned: Option<bool>,
}

/// Message Time Period
///
/// Filter and sort messages by time
pub enum MessageTimePeriod {
  Relative {
    /// Message id to search around
    ///
    /// Specifying 'nearby' ignores 'before', 'after' and 'sort'.
    /// It will also take half of limit rounded as the limits to each side.
    /// It also fetches the message ID specified.
    nearby: String,
  },
  Absolute {
    /// Message id before which messages should be fetched
    before: Option<String>,
    /// Message id after which messages should be fetched
    after: Option<String>,
    /// Message sort direction
    sort: Option<MessageSort>,
  },
}

pub fn get_message_system_user_ids(system: Option<&MessageSystem>) -> Vec<String> {
  let mut users = vec![];

  if let Some(system) = system {
    if let Some(text) = &system.text {
      // No user IDs in text message
      let _ = text;
    }
    if let Some(user_added) = &system.user_added {
      users.push(user_added.by.clone());
      users.push(user_added.id.clone());
    }
    if let Some(user_remove) = &system.user_remove {
      users.push(user_remove.by.clone());
      users.push(user_remove.id.clone());
    }
    if let Some(user_joined) = &system.user_joined {
      users.push(user_joined.id.clone());
    }
    if let Some(user_left) = &system.user_left {
      users.push(user_left.id.clone());
    }
    if let Some(user_kicked) = &system.user_kicked {
      users.push(user_kicked.id.clone());
    }
    if let Some(user_banned) = &system.user_banned {
      users.push(user_banned.id.clone());
    }
    if let Some(channel_renamed) = &system.channel_renamed {
      users.push(channel_renamed.by.clone());
    }
    if let Some(channel_description_changed) = &system.channel_description_changed {
      users.push(channel_description_changed.by.clone());
    }
    if let Some(channel_icon_changed) = &system.channel_icon_changed {
      users.push(channel_icon_changed.by.clone());
    }
    if let Some(channel_ownership_changed) = &system.channel_ownership_changed {
      users.push(channel_ownership_changed.from.clone());
      users.push(channel_ownership_changed.to.clone());
    }
    if let Some(message_pinned) = &system.message_pinned {
      users.push(message_pinned.by.clone());
    }
    if let Some(message_unpinned) = &system.message_unpinned {
      users.push(message_unpinned.by.clone());
    }
    if let Some(call_started) = &system.call_started {
      users.push(call_started.by.clone());
    }
  }

  users
}
