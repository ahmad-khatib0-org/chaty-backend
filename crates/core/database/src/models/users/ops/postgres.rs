use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::User;
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{CachedUserData, PostgresDb, UsersRepository};

#[async_trait()]
impl UsersRepository for PostgresDb {
  async fn users_create(&self, _ctx: Arc<Context>, user: &User) -> Result<(), DBError> {
    let path = "database.users.users_create".to_string();

    // Convert u32 badges to i32 and u64 timestamps to i64 for PostgreSQL compatibility
    let badges = user.badges.map(|b| b as i32);
    let suspended_until = user.suspended_until.map(|s| s as i64);
    let created_at = user.created_at as i64;
    let updated_at = user.updated_at as i64;

    let result: Result<_, _> = sqlx::query(
      "INSERT INTO users (id, username, email, password_hash, display_name, badges, 
       status_text, status_presence, profile_content, profile_background_id, 
       privileged, suspended_until, created_at, updated_at, verified)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password)
    .bind(&user.display_name)
    .bind(badges)
    .bind(&user.status_text)
    .bind(&user.status_presence)
    .bind(&user.profile_content)
    .bind(&user.profile_background_id)
    .bind(user.privileged)
    .bind(suspended_until)
    .bind(created_at)
    .bind(updated_at)
    .bind(user.verified)
    .execute(self.db())
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(err) => {
        let err_type = if err.to_string().contains("unique constraint") {
          ErrorType::ResourceExists
        } else {
          ErrorType::DatabaseError
        };

        Err(DBError {
          err_type,
          msg: format!("failed to create user: {}", err),
          path,
          ..Default::default()
        })
      }
    }
  }

  async fn users_get_auth_data(
    &self,
    _ctx: Arc<Context>,
    user_id: &str,
  ) -> Result<CachedUserData, DBError> {
    let path = "database.users.users_get_auth_data".to_string();

    let row: Result<_, _> = sqlx::query("SELECT id FROM users WHERE id = $1")
      .bind(user_id)
      .fetch_optional(self.db())
      .await;

    match row {
      Ok(Some(_)) => Ok(CachedUserData { ..Default::default() }),
      Ok(None) => Err(DBError {
        err_type: ErrorType::NotFound,
        msg: "user not found".to_string(),
        path,
        ..Default::default()
      }),
      Err(err) => Err(DBError {
        err_type: ErrorType::DatabaseError,
        msg: format!("failed to fetch user auth data: {}", err),
        path,
        ..Default::default()
      }),
    }
  }
}
