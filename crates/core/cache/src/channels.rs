use std::sync::Arc;

use chaty_proto::Channel;
use chaty_result::{
  context::Context,
  errors::{BoxedErr, DBError, ErrorType, InternalError},
};
use deadpool_redis::redis::AsyncCommands;

use crate::Cache;

impl Cache {
  pub async fn channels_get_by_id(&self, id: &str) -> Result<Option<Channel>, BoxedErr> {
    let path = "cache.channels.channels_get_by_id".to_string();
    let mut conn = self.get_conn(&path).await?;

    let ie = |err: BoxedErr, msg: &str| {
      InternalError::new(path.clone(), err, ErrorType::InternalError, false, msg.into())
    };

    let res: Option<String> = conn
      .get(self.channels_by_id_key(id))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get channels data from redis"))?;

    match res {
      Some(res) => {
        let chan: Channel = serde_json::from_str(&res)
          .map_err(|err| ie(Box::new(err), "failed to serialize channel"))?;
        Ok(Some(chan))
      }
      None => Ok(None),
    }
  }

  pub async fn channels_get_or_insert_by_id(
    &self,
    ctx: Arc<Context>,
    id: &str,
  ) -> Result<Channel, DBError> {
    let path = "cache.channels.channels_get_or_insert_by_id".to_string();
    let mut conn = self.get_conn(&path).await?;

    let ie =
      |err: BoxedErr, msg: &str| DBError::new(path.clone(), err, ErrorType::InternalError, msg);

    let res: Option<String> = conn
      .get(self.channels_by_id_key(id))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get channels data from redis"))?;

    match res {
      Some(res) => {
        let chan: Channel = serde_json::from_str(&res)
          .map_err(|err| ie(Box::new(err), "failed to deserialize channel"))?;
        Ok(chan)
      }
      None => {
        let chan = self.nosql_db.channels_get_by_id(ctx, id).await?;
        let mut conn = self.get_conn(&path).await?;

        let payload = serde_json::to_string(&chan).map_err(|err| {
          let msg = "failed to serialize channel".to_string();
          DBError::new(path.clone(), Box::new(err), ErrorType::JsonMarshal, msg)
        })?;

        let _: () = conn.set(self.channels_by_id_key(id), payload).await.map_err(|err| {
          let msg = "failed to insert channel in redis";
          DBError::new(&path, Box::new(err), ErrorType::DBInsertError, msg)
        })?;

        Ok(chan)
      }
    }
  }
}
