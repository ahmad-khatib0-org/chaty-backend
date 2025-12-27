use std::{collections::HashMap, sync::Arc};

use chaty_proto::User;
use tokio::sync::Mutex;

#[derive(Default, Debug)]
pub struct ReferenceDb {
  pub users: Arc<Mutex<HashMap<String, User>>>,
}
