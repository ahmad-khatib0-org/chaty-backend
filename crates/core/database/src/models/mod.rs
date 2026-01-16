mod channels;
mod helpers;
mod serve_members;
mod servers;
mod users;

use std::ops::Deref;

pub use channels::*;
pub use helpers::*;
pub use serve_members::*;
pub use servers::*;
pub use users::*;

#[cfg(feature = "postgres")]
use crate::PostgresDb;

#[cfg(feature = "scylladb")]
use crate::{DatabaseNoSql, ReferenceNoSqlDb, ScyllaDb};
use crate::{DatabaseSql, ReferenceSqlDb};

pub trait AbstractDatabaseSql: Sync + Send + UsersRepository {}

pub trait AbstractDatabaseNoSql:
  Sync + Send + ChannelsRepository + ServerMembersRepository + ServersRepository + HelpersRepository
{
}

impl AbstractDatabaseNoSql for ReferenceNoSqlDb {}
impl AbstractDatabaseSql for ReferenceSqlDb {}

#[cfg(feature = "scylladb")]
impl AbstractDatabaseNoSql for ScyllaDb {}

#[cfg(feature = "postgres")]
impl AbstractDatabaseSql for PostgresDb {}

impl Deref for DatabaseNoSql {
  type Target = dyn AbstractDatabaseNoSql;

  fn deref(&self) -> &Self::Target {
    match self {
      DatabaseNoSql::Reference(dummy) => dummy,
      #[cfg(feature = "scylladb")]
      DatabaseNoSql::Scylladb(scylla) => scylla,
    }
  }
}

impl Deref for DatabaseSql {
  type Target = dyn AbstractDatabaseSql;

  fn deref(&self) -> &Self::Target {
    match self {
      DatabaseSql::Reference(dummy) => dummy,
      #[cfg(feature = "postgres")]
      DatabaseSql::Postgres(postgres) => postgres,
    }
  }
}

pub trait EnumHelpers {
  fn to_string(&self) -> String {
    self.to_str().to_string()
  }

  fn to_str(&self) -> &'static str;

  fn from_optional_string(value: Option<String>) -> Option<Self>
  where
    Self: Sized;

  fn from_str(value: &str) -> Option<Self>
  where
    Self: Sized,
  {
    EnumHelpers::from_optional_string(Some(value.to_string()))
  }

  fn from_string(value: String) -> Option<Self>
  where
    Self: Sized,
  {
    EnumHelpers::from_optional_string(Some(value))
  }

  fn to_i32(&self) -> i32 {
    return 0;
  }
}
