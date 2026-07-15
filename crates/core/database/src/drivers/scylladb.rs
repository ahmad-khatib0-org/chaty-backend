use std::ops::Deref;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

#[derive(Debug)]
pub struct Prepared {
  pub servers: PreparedServers,
  pub channels: PreparedChannels,
  pub server_members: PreparedServerMembers,
  pub messages: PreparedMessages,
  pub server_bans: PreparedServerBans,
}

#[derive(Debug)]
pub struct PreparedServers {
  pub get_server_by_id: PreparedStatement,
  pub insert_server: PreparedStatement,
}

#[derive(Debug)]
pub struct PreparedChannels {
  pub insert_channel: PreparedStatement,
  pub insert_channel_by_user: PreparedStatement,
  pub groups_list_first_page: PreparedStatement,
  pub groups_list_next_page: PreparedStatement,
  pub get_channel_by_id: PreparedStatement,
}

#[derive(Debug)]
pub struct PreparedServerMembers {
  pub get_server_ids_by_user_id: PreparedStatement,
  pub get_server_member_by_id: PreparedStatement,
  pub get_server_members_by_ids: PreparedStatement,
  pub get_server_members_count: PreparedStatement,
  pub insert_server_member: PreparedStatement,
}

#[derive(Debug)]
pub struct PreparedMessages {
  pub get_messages_by_channel_id: PreparedStatement,
  pub get_messages_by_channel_id_gt: PreparedStatement, // newer than (id >)
  pub get_messages_by_channel_id_gte: PreparedStatement, // newer or equal (id >=)
  pub get_messages_by_channel_id_lt: PreparedStatement, // older than (id <)
  pub get_messages_by_channel_id_lte: PreparedStatement, // older or equal (id <=)
  pub get_messages_by_channel_id_range: PreparedStatement, // between (id > x AND id < y)
}

#[derive(Debug)]
pub struct PreparedServerBans {
  pub insert_server_ban: PreparedStatement,
  pub get_server_ban: PreparedStatement,
}

/// Scylladb implementation
#[derive(Debug)]
pub struct ScyllaDb {
  pub db: Session,
  pub prepared: Prepared,
}

impl Deref for ScyllaDb {
  type Target = Session;

  fn deref(&self) -> &Self::Target {
    &self.db
  }
}

impl ScyllaDb {
  pub fn db(&self) -> &Session {
    &self.db
  }
}
