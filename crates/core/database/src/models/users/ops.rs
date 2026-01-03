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
  /// Create a new token for a user
  async fn users_get_by_email(&self, ctx: Arc<Context>, email: &str) -> Result<User, DBError>;
  /// Gets user information about auth status, E,g if user registered
  /// with social account roles, ....
  async fn users_get_auth_data(
    &self,
    ctx: Arc<Context>,
    user_id: &str,
  ) -> Result<CachedUserData, DBError>;
  async fn tokens_create(&self, ctx: Arc<Context>, token: &Token) -> Result<(), DBError>;
}
