use std::ops::Deref;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

#[derive(Debug)]
pub struct PreparedChannels {
  pub insert_channel: PreparedStatement,
  pub insert_channel_by_user: PreparedStatement,
  pub groups_list_first_page: PreparedStatement,
  pub groups_list_next_page: PreparedStatement,
}

#[derive(Debug)]
pub struct Prepared {
  pub channels: PreparedChannels,
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
