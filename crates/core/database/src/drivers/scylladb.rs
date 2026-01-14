use std::ops::Deref;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

#[derive(Debug)]
pub struct Prepared {
  pub servers: PreparedServers,
  pub channels: PreparedChannels,
  pub server_members: PreparedServerMembers,
}

#[derive(Debug)]
pub struct PreparedServers {
  pub get_server_by_id: PreparedStatement,
}

#[derive(Debug)]
pub struct PreparedChannels {
  pub insert_channel: PreparedStatement,
  pub insert_channel_by_user: PreparedStatement,
  pub groups_list_first_page: PreparedStatement,
  pub groups_list_next_page: PreparedStatement,
  pub insert_channel_by_recipient: PreparedStatement,
}

#[derive(Debug)]
pub struct PreparedServerMembers {
  pub get_server_ids_by_user_id: PreparedStatement,
  pub get_server_member_by_id: PreparedStatement,
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
