mod init;

use std::{io::ErrorKind, sync::Arc};

use chaty_config::{config, Settings};
use chaty_database::{DatabaseInfoSql, DatabaseSql};
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::Pool as RedisPool;
use tokio::{
  spawn,
  sync::mpsc::{channel, Receiver, Sender},
};

use crate::controller::{otel::init_otel, Controller, ControllerArgs};

#[derive(Clone)]
pub struct Server {
  pub(crate) errors_send: Sender<InternalError>,
  pub(crate) config: Arc<Settings>,
  pub(crate) redis: Option<Arc<RedisPool>>,
  pub(crate) sql_db: Option<Arc<DatabaseSql>>,
}

impl Server {
  pub async fn new() -> Result<Self, BoxedErr> {
    let (tx, rx) = channel::<InternalError>(100);

    let srv =
      Server { errors_send: tx, config: Arc::new(Settings::default()), redis: None, sql_db: None };

    let srv_clone = srv.clone();
    spawn(async move { srv_clone.errors_listener(rx).await });

    Ok(srv)
  }

  pub async fn run(&mut self) -> Result<(), BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| InternalError {
      err_type: ErrorType::InternalError,
      temp: false,
      err,
      msg: msg.into(),
      path: "auth.server.run".into(),
    };

    let config = config().await;
    let sql_db =
      DatabaseInfoSql::Postgres { dsn: config.database.postgres.clone() }.connect().await.map_err(
        |err| ie(&err.clone(), Box::new(std::io::Error::new(ErrorKind::NotConnected, err))),
      )?;

    self.config = Arc::new(config);
    self.redis = Some(Arc::new(self.init_redis().await?));
    self.sql_db = Some(Arc::new(sql_db));

    let (_registry, metrics) = init_otel().map_err(|err| {
      ie(
        "failed to initialize OTEL",
        Box::new(std::io::Error::new(ErrorKind::Other, format!("{:?}", err))),
      )
    })?;

    let controller_args = {
      ControllerArgs {
        config: Arc::new(self.config.as_ref().clone()),
        redis_con: self.redis.as_ref().unwrap().clone(),
        sql_db: self.sql_db.as_ref().unwrap().clone(),
        metrics,
      }
    };

    let controller = Controller::new(controller_args).await;
    controller.run().await?;

    Ok(())
  }

  pub async fn errors_listener(&self, mut receiver: Receiver<InternalError>) {
    while let Some(msg) = receiver.recv().await {
      println!("received an internal error: {}", msg)
    }
  }
}
