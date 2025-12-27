mod users;

pub use users::*;

#[cfg(feature = "scylladb")]
use crate::{Database, ReferenceDb, ScyllaDb};

pub trait AbstractDatabase: Sync + Send + UsersRepository {}

impl AbstractDatabase for ReferenceDb {}

#[cfg(feature = "scylladb")]
impl AbstractDatabase for ScyllaDb {}

impl std::ops::Deref for Database {
  type Target = dyn AbstractDatabase;

  fn deref(&self) -> &Self::Target {
    match self {
      Database::Reference(dummy) => dummy,
      #[cfg(feature = "scylladb")]
      Database::Scylladb(scylla) => scylla,
    }
  }
}
