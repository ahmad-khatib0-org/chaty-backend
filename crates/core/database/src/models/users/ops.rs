mod reference;

#[cfg(feature = "scylladb")]
mod scylladb;

use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{context::Context, errors::DBError};

#[async_trait]
pub trait UsersRepository: Sync + Send {
  /// Insert a new user into database
  async fn users_create(&self, ctx: Arc<Context>, user: &User) -> Result<(), DBError>;
}
