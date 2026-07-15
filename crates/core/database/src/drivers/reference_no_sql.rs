use std::{collections::HashMap, sync::Arc};

use chaty_proto::{Channel, Message, Server, ServerBan, ServerMember, ServerMemberCompositeKey};
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct ReferenceNoSqlDb {
  pub channels: Arc<Mutex<HashMap<String, Channel>>>,
  pub server_members: Arc<Mutex<HashMap<String, ServerMember>>>,
  pub servers: Arc<Mutex<HashMap<String, Server>>>,
  pub messages: Arc<Mutex<HashMap<String, Message>>>,
  pub server_bans: Arc<Mutex<HashMap<ServerMemberCompositeKey, ServerBan>>>,
}
