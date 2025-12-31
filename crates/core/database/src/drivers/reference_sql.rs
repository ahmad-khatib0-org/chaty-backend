use std::{collections::HashMap, sync::Arc};

use chaty_proto::User;
use tokio::sync::Mutex;

use crate::Token;

#[derive(Default, Debug)]
pub struct ReferenceSqlDb {
  pub users: Arc<Mutex<HashMap<String, User>>>,
  pub tokens: Arc<Mutex<HashMap<String, Token>>>,
}
