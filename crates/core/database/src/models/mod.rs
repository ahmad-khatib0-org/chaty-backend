mod users;

pub use users::*;

#[cfg(feature = "postgres")]
use crate::PostgresDb;
#[cfg(feature = "scylladb")]
use crate::{DatabaseNoSql, ReferenceNoSqlDb, ScyllaDb};
use crate::{DatabaseSql, ReferenceSqlDb};

pub trait AbstractDatabaseSql: Sync + Send + UsersRepository {}

pub trait AbstractDatabaseNoSql: Sync + Send {}

impl AbstractDatabaseNoSql for ReferenceNoSqlDb {}
impl AbstractDatabaseSql for ReferenceSqlDb {}

#[cfg(feature = "scylladb")]
impl AbstractDatabaseNoSql for ScyllaDb {}

#[cfg(feature = "postgres")]
impl AbstractDatabaseSql for PostgresDb {}

impl std::ops::Deref for DatabaseNoSql {
  type Target = dyn AbstractDatabaseNoSql;

  fn deref(&self) -> &Self::Target {
    match self {
      DatabaseNoSql::Reference(dummy) => dummy,
      #[cfg(feature = "scylladb")]
      DatabaseNoSql::Scylladb(scylla) => scylla,
    }
  }
}

impl std::ops::Deref for DatabaseSql {
  type Target = dyn AbstractDatabaseSql;

  fn deref(&self) -> &Self::Target {
    match self {
      DatabaseSql::Reference(dummy) => dummy,
      #[cfg(feature = "postgres")]
      DatabaseSql::Postgres(postgres) => postgres,
    }
  }
}
