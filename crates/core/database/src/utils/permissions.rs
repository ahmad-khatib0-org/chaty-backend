use std::{borrow::Cow, collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chaty_permission::{
  ChannelType, Override, PermissionQuery, PermissionValue, RelationshipStatus,
  DEFAULT_PERMISSION_DIRECT_MESSAGE,
};
use chaty_proto::{Server, ServerMember, User, UserRelationshipStatus};
use chaty_result::context::Context;

use crate::{ChannelDB, DatabaseNoSql, DatabaseSql, EnumHelpers};

/// Permissions calculator
pub struct DatabasePermissionQuery<'a> {
  ctx: Arc<Context>,
  nosql_db: &'a DatabaseNoSql,
  sql_db: &'a DatabaseSql,

  perspective: &'a User,
  user: Option<Cow<'a, User>>,
  server: Option<Cow<'a, Server>>,
  channel: Option<Cow<'a, ChannelDB>>,
  member: Option<Cow<'a, ServerMember>>,

  cached_user_permission: Option<PermissionValue>,
  cached_mutual_connection: Option<bool>,
  cached_permission: Option<u64>,
}

#[async_trait]
impl PermissionQuery for DatabasePermissionQuery<'_> {
  /// Is our perspective user privileged?
  async fn is_privileged(&mut self) -> bool {
    self.perspective.privileged
  }

  /// Is our perspective user a bot?
  async fn is_bot(&mut self) -> bool {
    self.perspective.bot.is_some()
  }

  /// Is our perspective user and the currently selected user the same?
  async fn are_the_users_same(&mut self) -> bool {
    if let Some(other_user) = &self.user {
      self.perspective.id == other_user.id
    } else {
      false
    }
  }

  /// Get the relationship with have with the currently selected user
  async fn user_relationship(&mut self) -> RelationshipStatus {
    if let Some(other) = &self.user {
      if self.perspective.id == other.id {
        return RelationshipStatus::User;
      } else if let Some(bot) = &other.bot {
        // For the purposes of permissions checks,
        // assume owner is the same as bot
        if self.perspective.id == bot.owner {
          return RelationshipStatus::User;
        }
      }

      for entry in &self.perspective.relations {
        if entry.id == other.id {
          return match entry.status() {
            UserRelationshipStatus::None => RelationshipStatus::None,
            UserRelationshipStatus::User => RelationshipStatus::User,
            UserRelationshipStatus::Friend => RelationshipStatus::Friend,
            UserRelationshipStatus::Outgoing => RelationshipStatus::Outgoing,
            UserRelationshipStatus::Incoming => RelationshipStatus::Incoming,
            UserRelationshipStatus::Blocked => RelationshipStatus::Blocked,
            UserRelationshipStatus::BlockedOther => RelationshipStatus::BlockedOther,
          };
        }
      }
    }

    RelationshipStatus::None
  }

  /// Whether the currently selected user is a bot
  async fn user_is_bot(&mut self) -> bool {
    if let Some(other_user) = &self.user {
      other_user.bot.is_some()
    } else {
      false
    }
  }

  /// Do we have a mutual connection with the currently selected user?
  async fn have_mutual_connection(&mut self) -> bool {
    if let Some(value) = self.cached_mutual_connection {
      return value;
    }

    let Some(user) = &self.user else {
        return false;
    };

    let (p_servers, u_servers) = tokio::join!(
      self.nosql_db.server_members_get_server_ids_by_user_id(&self.perspective.id),
      self.nosql_db.server_members_get_server_ids_by_user_id(&user.id)
    );

    let p_server_ids = p_servers.unwrap_or_default();
    let u_server_ids = u_servers.unwrap_or_default();
    let p_server_set: HashSet<_> = p_server_ids.into_iter().collect();

    if u_server_ids.iter().any(|id| p_server_set.contains(id)) {
      self.cached_mutual_connection = Some(true);
      return true;
    }

    let channel_types = vec![ChannelType::DirectMessage.to_str(), ChannelType::Group.to_str()];
    let (p_chans, u_chans) = tokio::join!(
      self.nosql_db.channels_get_channels_ids_by_user_id(&self.perspective.id, &channel_types),
      self.nosql_db.channels_get_channels_ids_by_user_id(&user.id, &channel_types)
    );

    let p_chan_ids = p_chans.unwrap_or_default();
    let u_chan_ids = u_chans.unwrap_or_default();

    let p_chan_set: HashSet<_> = p_chan_ids.into_iter().collect();
    if u_chan_ids.iter().any(|id| p_chan_set.contains(id)) {
      self.cached_mutual_connection = Some(true);
      return true;
    }

    self.cached_mutual_connection = Some(false);
    false
  }

  // * For calculating server permission

  /// Is our perspective user the server's owner?
  async fn is_server_owner(&mut self) -> bool {
    if let Some(server) = &self.server {
      server.owner_id == self.perspective.id
    } else {
      false
    }
  }

  /// Is our perspective user a member of the server?
  async fn is_server_member(&mut self) -> bool {
    if let Some(server) = &self.server {
      if self.member.is_some() {
        true
      } else if let Ok(member) =
        self.nosql_db.server_members_get_member(&server.id, &self.perspective.id).await
      {
        self.member = Some(Cow::Owned(member));
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Get default server permission
  async fn get_default_server_permissions(&mut self) -> u64 {
    if let Some(server) = &self.server {
      server.default_permissions as u64
    } else {
      0
    }
  }

  async fn get_our_server_role_overrides(&mut self) -> Vec<Override> {
    if let Some(server) = &self.server {
      let member_roles = self.member.as_ref().map(|m| m.roles.clone()).unwrap_or_default();

      let mut roles = server
        .roles
        .iter()
        .filter(|(id, _)| member_roles.contains(id))
        .map(|(_, role)| {
          let v: Override = role.permissions.unwrap_or_default().into();
          (role.rank, v)
        })
        .collect::<Vec<(i64, Override)>>();

      roles.sort_by(|a, b| b.0.cmp(&a.0));
      roles.into_iter().map(|(_, v)| v).collect()
    } else {
      vec![]
    }
  }

  /// Is our perspective user timed out on this server?
  async fn is_timed_out(&mut self) -> bool {
    if let Some(member) = &self.member {
      self.nosql_db.server_members_is_member_in_timeout(&member)
    } else {
      false
    }
  }

  async fn have_publish_overwrites(&mut self) -> bool {
    if let Some(member) = &self.member {
      member.can_publish
    } else {
      true
    }
  }

  async fn have_receive_overwrites(&mut self) -> bool {
    if let Some(member) = &self.member {
      member.can_receive
    } else {
      true
    }
  }

  // * For calculating channel permission

  async fn get_channel_type(&mut self) -> ChannelType {
    if let Some(channel) = &self.channel {
      ChannelType::from_optional_string(Some(channel.channel_type.clone()))
        .unwrap_or(ChannelType::Unknown)
    } else {
      ChannelType::Unknown
    }
  }

  async fn get_default_channel_permissions(&mut self) -> Override {
    let channel = match &self.channel {
      Some(chan) => chan.as_ref(),
      None => return Default::default(),
    };

    match ChannelType::from_str(&channel.channel_type).unwrap_or(ChannelType::Unknown) {
      ChannelType::Group => {
        if let Some(group) = &channel.group {
          Override {
            allow: group.permissions.unwrap_or(*DEFAULT_PERMISSION_DIRECT_MESSAGE as i64) as u64,
            deny: 0,
          }
        } else {
          Default::default()
        }
      }
      ChannelType::ServerChannel => {
        if let Some(text) = &channel.text {
          text.default_permissions.as_ref().map(|p| p.into()).unwrap_or_default()
        } else {
          Default::default()
        }
      }
      _ => Default::default(),
    }
  }

  async fn get_our_channel_role_overrides(&mut self) -> Vec<Override> {
    let channel = match &self.channel {
      Some(chan) => chan.as_ref(),
      None => return vec![],
    };

    let (role_permissions, server) = match (&channel.text, &self.server) {
      (Some(text), Some(srv)) => (&text.role_permissions, srv.as_ref()),
      _ => return vec![],
    };

    let member_roles = self.member.as_ref().map(|m| m.roles.clone()).unwrap_or_default();

    // Filter and sort role overrides
    let mut roles: Vec<(i64, Override)> = role_permissions
      .iter()
      .filter(|(role_id, _)| member_roles.contains(role_id))
      .filter_map(|(role_id, permission)| {
        server.roles.get(role_id).map(|role| {
          let v: Override = permission.into();
          (role.rank, v)
        })
      })
      .collect();

    // Sort by rank descending (highest rank first)
    roles.sort_by(|a, b| b.0.cmp(&a.0));
    roles.into_iter().map(|(_, v)| v).collect()
  }

  async fn is_channel_owner(&mut self) -> bool {
    let channel = match &self.channel {
      Some(chan) => chan.as_ref(),
      None => return false,
    };

    if let Some(group) = &channel.group {
      return group.user_id == self.perspective.id;
    }
    if let Some(saved) = &channel.saved {
      return saved.user_id == self.perspective.id;
    }
    false
  }

  async fn is_part_of_the_channel(&mut self) -> bool {
    let channel = match &self.channel {
      Some(chan) => chan.as_ref(),
      None => return false,
    };

    if let Some(direct) = &channel.direct {
      return direct.recipients.contains(&self.perspective.id);
    }
    if let Some(group) = &channel.group {
      return group.recipients.contains(&self.perspective.id);
    }
    false
  }

  async fn set_recipient_as_user(&mut self) {
    let channel = match &self.channel {
      Some(chan) => chan.as_ref(),
      None => return,
    };

    if let Some(direct) = &channel.direct {
      // Find the OTHER user in the DM (not perspective)
      if let Some(recipient_id) =
        direct.recipients.iter().find(|recipient| recipient != &&self.perspective.id)
      {
        if let Ok(user) = self.sql_db.users_get_by_id(self.ctx.clone(), recipient_id).await {
          self.user = Some(Cow::Owned(user));
        }
      } else {
        panic!("database.utils.permission.set_recipient_as_user: Missing recipient for DM");
      }
    } else {
      unimplemented!();
    }
  }

  async fn set_server_from_channel(&mut self) {
    let channel = match &self.channel {
      Some(cow) => cow.as_ref(),
      None => return,
    };

    if let Some(text) = &channel.text {
      let server_id = &text.server_id;

      // Check if already cached (using .as_ref() to handle both Cow variants)
      if let Some(known_server) = self.server.as_ref() {
        if server_id == &known_server.id {
          return; // Already cached
        }
      }

      if let Ok(server) = self.nosql_db.servers_get_server_by_id(server_id).await {
        self.server = Some(Cow::Owned(server));
      }
    } else {
      // Not a server channel
      unimplemented!();
    }
  }
}
