use chaty_proto::UserVoiceState;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use deadpool_redis::redis::AsyncCommands;

use crate::Cache;

impl Cache {
  pub async fn voice_get_channel_members(&self, channel_id: &str) -> Result<Vec<String>, BoxedErr> {
    let path = "cache.voice.voice_get_channel_members".to_string();
    let mut conn = self.get_conn(&path).await?;

    let ie = |err: BoxedErr, msg: &str| {
      InternalError::new(path.clone(), err, ErrorType::InternalError, false, msg.into())
    };

    let members: Vec<String> = conn
      .smembers(format!("vc_members:{channel_id}"))
      .await
      .map_err(|err| ie(Box::new(err), "failed to get voice channel members from redis"))?;

    Ok(members)
  }

  pub async fn voice_get_user_channel_state(
    &self,
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
  ) -> Result<Option<UserVoiceState>, BoxedErr> {
    let path = "cache.voice.voice_get_user_channel_state".to_string();
    let mut conn = self.get_conn(&path).await?;

    let unique_key = format!("{}:{}", user_id, server_id.unwrap_or(channel_id));

    let keys = vec![
      format!("joined_at:{unique_key}"),
      format!("is_publishing:{unique_key}"),
      format!("is_receiving:{unique_key}"),
      format!("screensharing:{unique_key}"),
      format!("camera:{unique_key}"),
    ];

    let results: Vec<Option<String>> = conn.mget(keys).await.map_err(|err| {
      let msg = "failed to get voice state from redis".into();
      InternalError::new(path.clone(), Box::new(err), ErrorType::InternalError, false, msg)
    })?;

    let (joined_at, is_publishing, is_receiving, screensharing, camera) = match results.as_slice() {
      [j, p, r, s, c] => (j, p, r, s, c),
      _ => {
        return Err(Box::new(InternalError::new(
          path,
          Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid number of results",
          )),
          ErrorType::InternalError,
          false,
          "mget returned wrong number of results".into(),
        )));
      }
    };

    match (
      joined_at.as_ref(),
      is_publishing.as_ref(),
      is_receiving.as_ref(),
      screensharing.as_ref(),
      camera.as_ref(),
    ) {
      (
        Some(joined_at_str),
        Some(is_publishing_str),
        Some(is_receiving_str),
        Some(screensharing_str),
        Some(camera_str),
      ) => {
        // Parse joined_at as i64 (milliseconds)
        let joined_at_ms = joined_at_str.parse::<i64>().map_err(|err| {
          let msg = "failed to parse joined_at".into();
          InternalError::new(path.clone(), Box::new(err), ErrorType::InternalError, false, msg)
        })?;

        // Parse booleans (assuming they're stored as "true"/"false" or "1"/"0")
        let parse_bool = |s: &str| -> Result<bool, BoxedErr> {
          match s {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(Box::new(InternalError::new(
              path.clone(),
              Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid boolean value: {}", s),
              )),
              ErrorType::InternalError,
              false,
              "failed to parse boolean".into(),
            ))),
          }
        };

        let is_publishing = parse_bool(is_publishing_str)?;
        let is_receiving = parse_bool(is_receiving_str)?;
        let screensharing = parse_bool(screensharing_str)?;
        let camera = parse_bool(camera_str)?;

        Ok(Some(UserVoiceState {
          id: user_id.to_string(),
          joined_at: joined_at_ms,
          is_receiving,
          is_publishing,
          screensharing,
          camera,
        }))
      }
      _ => Ok(None),
    }
  }
}
