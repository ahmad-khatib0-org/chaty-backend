mod reference_sql;

#[cfg(feature = "postgres")]
mod postgres;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{context::Context, errors::DBError};

use crate::{CachedUserData, Token};

#[async_trait]
pub trait UsersRepository: Sync + Send {
  /// Insert a new user into database
  async fn users_create(&self, ctx: Arc<Context>, user: &User) -> Result<(), DBError>;
  /// Get user by email
  async fn users_get_by_email(&self, ctx: Arc<Context>, email: &str) -> Result<User, DBError>;
  /// Get user by id
  async fn users_get_by_id(&self, ctx: Arc<Context>, user_id: &str) -> Result<User, DBError>;
  /// Gets user information about auth status, E,g if user registered
  /// with social account roles, ....
  async fn users_get_auth_data(
    &self,
    ctx: Arc<Context>,
    user_id: &str,
  ) -> Result<CachedUserData, DBError>;
  /// Update user password
  async fn users_update_password(
    &self,
    ctx: Arc<Context>,
    user_id: &str,
    password_hash: &str,
  ) -> Result<(), DBError>;
  async fn tokens_create(&self, ctx: Arc<Context>, token: &Token) -> Result<(), DBError>;
  /// Get token by token string
  async fn tokens_get_by_token(&self, ctx: Arc<Context>, token: &str) -> Result<Token, DBError>;
  /// Mark token as used
  async fn tokens_mark_as_used(&self, ctx: Arc<Context>, token_id: &str) -> Result<(), DBError>;
}
