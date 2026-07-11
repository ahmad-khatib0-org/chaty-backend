use std::{collections::HashMap, sync::Arc};

use chaty_permission::DEFAULT_PERMISSION_SERVER;
use chaty_proto::{Channel, ChannelsCreateRequest, Server, ServersCreateRequest, User};
use chaty_result::{
  context::Context,
  errors::{AppError, OptionalParams},
};
use chaty_utils::time::time_get_millis;
use serde_json::{json, Value};
use tonic::Code;

use crate::models::channels::channels_create::channels_create_presave;

static SERVERS_NAME_MAX_LENGHT: usize = 32;
static SERVERS_NAME_MIN_LENGHT: usize = 1;
static SERVERS_DESCRIPTION_MAX_LENGHT: usize = 1024;

pub fn servers_create_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &ServersCreateRequest,
) -> Result<(), AppError> {
  let ae = |id: &str, params: OptionalParams| {
    return AppError::new(ctx.clone(), path, id, params, "", Code::InvalidArgument.into(), None);
  };

  let name = req.name.trim().len();
  let desc = req.description().trim().len();

  if name > SERVERS_NAME_MAX_LENGHT || name < SERVERS_NAME_MIN_LENGHT {
    let params = HashMap::from([
      ("Min".to_string(), SERVERS_NAME_MIN_LENGHT.into()),
      ("Max".to_string(), SERVERS_NAME_MAX_LENGHT.into()),
    ]);
    return Err(ae("servers.name.length", Some(params)));
  }

  if desc > SERVERS_NAME_MAX_LENGHT {
    let params = HashMap::from([("Max".to_string(), SERVERS_DESCRIPTION_MAX_LENGHT.into())]);
    return Err(ae("servers.description.max_length", Some(params)));
  }

  Ok(())
}

pub fn servers_create_presave(
  request: ServersCreateRequest,
  owner: &User,
  create_default_channels: bool,
) -> (Server, Vec<Channel>) {
  let mut srv = Server {
    id: ulid::Ulid::new().to_string(),
    owner_id: owner.id.to_string(),
    name: request.name,
    description: request.description,
    channels: vec![],
    nsfw: request.nsfw.unwrap_or(false),
    default_permissions: *DEFAULT_PERMISSION_SERVER as i64,

    analytics: false,
    banner: None,
    categories: vec![],
    discoverable: false,
    flags: None,
    icon: None,
    roles: HashMap::new(),
    system_messages: None,
    stats: None,
    created_at: time_get_millis(),
    updated_at: time_get_millis(),
  };

  let channels: Vec<Channel> = if create_default_channels {
    let channel = ChannelsCreateRequest {
      channel_type: "text".to_string(),
      name: "General".to_string(),
      ..Default::default()
    };
    vec![channels_create_presave(owner, &srv, &channel, None, None)]
  } else {
    vec![]
  };

  srv.channels = channels.iter().map(|c| c.id.to_string()).collect();
  (srv, channels)
}

// create an auditable request to be saved
pub fn servers_create_auditable(server: &ServersCreateRequest) -> Value {
  json!({ "name": server.name, "description": server.description, "nsfw": server.nsfw })
}
