use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{context::Context, errors::DBError};

use crate::{ScyllaDb, UsersRepository};

#[async_trait()]
impl UsersRepository for ScyllaDb {
  async fn users_create(&self, ctx: Arc<Context>, user: &User) -> Result<(), DBError> {
    let path = "database.users.insert_user".to_string();

    Ok(())
  }
}
