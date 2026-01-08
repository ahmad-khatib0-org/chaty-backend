use std::{collections::HashMap, sync::Arc};

use chaty_proto::{channel::ChannelData, Channel, ChannelGroup, GroupsCreateRequest, Timestamp};
use chaty_result::{
  context::Context,
  errors::{AppError, OptionalParams},
};
use chaty_utils::time::time_get_millis;
use serde_json::{json, Value};
use tonic::Code;
use ulid::Ulid;

use crate::models::groups::{
  GROUPS_DESCRIPTION_MAX_LENGTH, GROUPS_NAME_MAX_LENGTH, GROUPS_NAME_MIN_LENGTH,
};

pub fn groups_create_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &GroupsCreateRequest,
) -> Result<(), AppError> {
  let ae = |id: &str, params: OptionalParams| {
    return AppError::new(ctx.clone(), path, id, params, "", Code::InvalidArgument.into(), None);
  };

  // Validate group name
  if req.name.trim().is_empty() {
    return Err(ae("groups.name.required", None));
  }
  if req.name.len() < GROUPS_NAME_MIN_LENGTH || req.name.len() > GROUPS_NAME_MAX_LENGTH {
    let params = HashMap::from([
      ("Min".to_string(), GROUPS_NAME_MIN_LENGTH.into()),
      ("Max".to_string(), GROUPS_NAME_MAX_LENGTH.into()),
    ]);
    return Err(ae("groups.name.length", Some(params)));
  }

  // Validate description if provided
  if let Some(desc) = &req.description {
    if desc.len() > GROUPS_DESCRIPTION_MAX_LENGTH {
      let params = HashMap::from([("Max".to_string(), GROUPS_DESCRIPTION_MAX_LENGTH.into())]);
      return Err(ae("groups.description.length", Some(params)));
    }
  }

  // Validate recipients
  // if req.recipients.is_empty() {
  //   return Err(ae("groups.recipients.required", None));
  // }

  Ok(())
}

/// Pre-save function to populate Channel from GroupsCreateRequest
pub async fn groups_create_pre_save(
  ctx: Arc<Context>,
  path: &str,
  user_id: &str,
  req: &GroupsCreateRequest,
) -> Result<Channel, AppError> {
  let _ie = |id: &str| {
    return AppError::new(ctx.clone(), path, id, None, "", Code::Internal.into(), None);
  };

  let now_millis = time_get_millis();
  let now_seconds = (now_millis / 1000) as i64;
  let now_nanos = ((now_millis % 1000) * 1_000_000) as i32;

  let group = ChannelGroup {
    user_id: user_id.to_string(),
    name: req.name.clone(),
    description: req.description.clone(),
    recipients: req.recipients.clone(),
    icon: None,
    last_message_id: None,
    permissions: None,
    nsfw: req.nsfw.unwrap_or_default(),
  };

  Ok(Channel {
    id: Ulid::new().to_string(),
    channel_type: "group".to_string(),
    voice_max_users: None,
    created_at: Some(Timestamp { seconds: now_seconds, nanos: now_nanos }),
    updated_at: Some(Timestamp { seconds: now_seconds, nanos: now_nanos }),
    channel_data: Some(ChannelData::Group(group)),
  })
}

/// Create an auditable request to be saved
pub fn groups_create_auditable(req: &GroupsCreateRequest) -> Value {
  json!({
    "name": req.name,
    "recipients_count": req.recipients.len(),
    "nsfw": req.nsfw
  })
}
