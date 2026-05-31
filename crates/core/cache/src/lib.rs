pub mod channels;
mod helpers;
mod init;
pub mod presence;
pub mod users;

use std::{
  io::{Error, ErrorKind},
  sync::Arc,
};

use chaty_config::{config, Settings};
use chaty_database::{DatabaseInfoNoSql, DatabaseInfoSql, DatabaseNoSql, DatabaseSql};
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::Pool;

use crate::init::init_redis;

#[derive(Debug)]
pub struct Cache {
  pub(crate) nosql_db: Arc<DatabaseNoSql>,
  pub(crate) sql_db: Arc<DatabaseSql>,
  pub(crate) config: Arc<Settings>,
  pub(crate) redis: Arc<Pool>,
}

impl Cache {
  pub async fn new() -> Result<Self, BoxedErr> {
    let config = config().await;
    let redis = init_redis(&config).await?;

    let ie = |err: BoxedErr, msg: &str| {
      let path = "cache.new".to_string();
      Box::new(InternalError::new(path, err, ErrorType::InternalError, false, msg.into()))
    };

    let nosql_db = DatabaseInfoNoSql::ScyllaDb {
      uri: config.database.scylladb.clone(),
      keyspace: config.database.db_name.clone(),
    }
    .connect()
    .await
    .map_err(|err| {
      ie(Box::new(Error::new(ErrorKind::NotConnected, err)), "failed to connect to nosql db")
    })?;

    let dsn = config.database.postgres.clone();
    let sql_db = DatabaseInfoSql::Postgres { dsn }.connect().await.map_err(|err| {
      ie(Box::new(Error::new(ErrorKind::NotConnected, err)), "failed to connect to sql db")
    })?;

    Ok(Self {
      nosql_db: Arc::new(nosql_db),
      sql_db: Arc::new(sql_db),
      redis: Arc::new(redis),
      config: Arc::new(config),
    })
  }
}
