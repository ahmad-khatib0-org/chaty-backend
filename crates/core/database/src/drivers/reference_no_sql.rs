use std::{collections::HashMap, sync::Arc};

use chaty_proto::{Channel, ServerMember};
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct ReferenceNoSqlDb {
  pub channels: Arc<Mutex<HashMap<String, Channel>>>,
  pub server_members: Arc<Mutex<HashMap<String, ServerMember>>>,
}
