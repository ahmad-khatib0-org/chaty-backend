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

  async fn users_get_auth_data(
    &self,
    _ctx: Arc<Context>,
    _user_id: &str,
  ) -> Result<CachedUserData, DBError> {
    Ok(CachedUserData { ..Default::default() })
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

  async fn users_get_by_email(&self, _ctx: Arc<Context>, email: &str) -> Result<User, DBError> {
    let users = self.users.lock().await;
    let path = "database.users.users_get_by_email".to_string();
    let user = users.iter().find(|u| u.1.email == email);

    match user {
      Some(user) => Ok(user.1.clone()),
      None => {
        let msg = "user is not exists".to_string();
        Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
      }
    }
  }

  async fn tokens_get_by_token(&self, _ctx: Arc<Context>, token: &str) -> Result<Token, DBError> {
    let tokens = self.tokens.lock().await;
    let path = "database.users.tokens_get_by_token".to_string();
    let token_obj = tokens.iter().find(|t| t.1.token == token);

    match token_obj {
      Some(token_obj) => Ok(token_obj.1.clone()),
      None => {
        let msg = "token not found".to_string();
        Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
      }
    }
  }

  async fn tokens_mark_as_used(&self, _ctx: Arc<Context>, token_id: &str) -> Result<(), DBError> {
    let mut tokens = self.tokens.lock().await;
    let path = "database.users.tokens_mark_as_used".to_string();

    if let Some(token) = tokens.get_mut(token_id) {
      token.used = true;
      Ok(())
    } else {
      let msg = "token not found".to_string();
      Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
    }
  }
}
