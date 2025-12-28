use std::time::Duration;

use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::{Config, Pool, PoolConfig, Runtime, Timeouts};

use super::Server;

impl Server {
  pub async fn init_redis(&self) -> Result<Pool, BoxedErr> {
    let url = self.config.database.dragonfly.clone();

    let mut cfg = Config::from_url(url);

    cfg.pool = Some(PoolConfig {
      max_size: 32,
      timeouts: Timeouts {
        wait: Some(Duration::from_secs(5)),     // wait for free con
        create: Some(Duration::from_secs(2)),   // creating new con timeout
        recycle: Some(Duration::from_secs(30)), // con recycle timeout
      },
      ..Default::default()
    });

    Ok(cfg.create_pool(Some(Runtime::Tokio1)).map_err(|err| InternalError {
      temp: false,
      err_type: ErrorType::InternalError,
      err: Box::new(err),
      msg: "failed to create a redis pool".into(),
      path: "auth.server.init_redis".into(),
    })?)
  }
}
