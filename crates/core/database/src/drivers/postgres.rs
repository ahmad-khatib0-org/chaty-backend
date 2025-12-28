use std::ops::Deref;

use sqlx::{Pool, Postgres as PostgresClient};

/// Postgres implementation
#[derive(Debug)]
pub struct PostgresDb(pub Pool<PostgresClient>);

impl Deref for PostgresDb {
  type Target = Pool<PostgresClient>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl PostgresDb {
  pub fn db(&self) -> &Pool<PostgresClient> {
    &self.0
  }
}

