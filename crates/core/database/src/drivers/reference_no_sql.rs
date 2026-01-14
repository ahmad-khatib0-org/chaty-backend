use std::{collections::HashMap, sync::Arc};

use chaty_proto::{Channel, Server, ServerMember};
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct ReferenceNoSqlDb {
  pub channels: Arc<Mutex<HashMap<String, Channel>>>,
  pub server_members: Arc<Mutex<HashMap<String, ServerMember>>>,
  pub servers: Arc<Mutex<HashMap<String, Server>>>,
}
