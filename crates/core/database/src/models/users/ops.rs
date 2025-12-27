mod reference;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{AppError, Context};

#[async_trait]
pub trait UsersRepository: Sync + Send {
  /// Insert a new user into database
  async fn insert_user(&self, ctx: Arc<Context>, user: &User) -> Result<(), AppError>;
}
