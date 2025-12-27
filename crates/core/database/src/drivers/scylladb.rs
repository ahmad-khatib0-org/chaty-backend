use std::ops::Deref;

use scylla::client::session::Session;

/// Scylladb implementation
#[derive(Debug)]
pub struct ScyllaDb(pub Session);

impl Deref for ScyllaDb {
  type Target = Session;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl ScyllaDb {
  pub fn db(&self) -> &Session {
    &self.0
  }
}
