mod reference_sql;

#[cfg(feature = "postgres")]
mod postgres;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{context::Context, errors::DBError};

use crate::CachedUserData;

#[async_trait]
pub trait UsersRepository: Sync + Send {
  /// Insert a new user into database
  async fn users_create(&self, ctx: Arc<Context>, user: &User) -> Result<(), DBError>;
  /// Gets user information about auth status, E,g if user registered
  /// with social account roles, ....
  async fn users_get_auth_data(
    &self,
    ctx: Arc<Context>,
    user_id: &str,
  ) -> Result<CachedUserData, DBError>;
}
