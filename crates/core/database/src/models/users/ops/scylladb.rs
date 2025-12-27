use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{AppError, Context};

use crate::{ScyllaDb, UsersRepository};

#[async_trait()]
impl UsersRepository for ScyllaDb {
  async fn insert_user(&self, ctx: Arc<Context>, user: &User) -> Result<(), AppError> {
    let path = "database.users.insert_user";

    Ok(())
  }
}
