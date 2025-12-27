use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{AppError, Context, ERROR_ID_ALREADY_EXISTS};
use tonic::Code;

use crate::{ReferenceDb, UsersRepository};

#[async_trait()]
impl UsersRepository for ReferenceDb {
  async fn insert_user(&self, ctx: Arc<Context>, user: &User) -> Result<(), AppError> {
    let mut users = self.users.lock().await;
    let path = "database.users.insert_user";

    if users.contains_key(&user.id) {
      let id = ERROR_ID_ALREADY_EXISTS.to_string();
      Err(AppError::new(ctx, path, id, None, "", Code::AlreadyExists.into(), None))
    } else {
      users.insert(user.id.to_string(), user.clone());
      Ok(())
    }
  }
}
