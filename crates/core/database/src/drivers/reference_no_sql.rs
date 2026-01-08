use std::{collections::HashMap, sync::Arc};

use chaty_proto::Channel;
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct ReferenceNoSqlDb {
  pub channels: Arc<Mutex<HashMap<String, Channel>>>,
}
