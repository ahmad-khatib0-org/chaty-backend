use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{CachedUserData, ReferenceSqlDb, Token, UsersRepository};

#[async_trait()]
impl UsersRepository for ReferenceSqlDb {
  async fn users_create(&self, _ctx: Arc<Context>, user: &User) -> Result<(), DBError> {
    let mut users = self.users.lock().await;
    let path = "database.users.insert_user".to_string();

    if users.contains_key(&user.id) {
      let msg = "user already exists".to_string();
      Err(DBError { err_type: ErrorType::ResourceExists, msg, path, ..Default::default() })
    } else {
      users.insert(user.id.to_string(), user.clone());
      Ok(())
    }
  }

  async fn tokens_create(&self, _ctx: Arc<Context>, token: &Token) -> Result<(), DBError> {
    let mut tokens = self.tokens.lock().await;
    let path = "database.users.tokens_create".to_string();

    if tokens.contains_key(&token.id) {
      let msg = "token already exists".to_string();
      Err(DBError { err_type: ErrorType::ResourceExists, msg, path, ..Default::default() })
    } else {
      tokens.insert(token.id.to_string(), token.clone());
      Ok(())
    }
  }

  async fn users_get_auth_data(
    &self,
    _ctx: Arc<Context>,
    _user_id: &str,
  ) -> Result<CachedUserData, DBError> {
    Ok(CachedUserData { ..Default::default() })
  }
}
