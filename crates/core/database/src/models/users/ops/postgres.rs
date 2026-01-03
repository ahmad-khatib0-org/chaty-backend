use std::sync::Arc;

use async_trait::async_trait;
use chaty_proto::{User, UserStatus};
use chaty_result::{
  context::Context,
  errors::{DBError, ErrorType},
};

use crate::{CachedUserData, EnumHelpers, PostgresDb, Token, TokenType, UsersRepository};

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

        let msg = format!("failed to create user: {}", err);
        Err(DBError { err_type, msg, path, err: Box::new(err) })
      }
    }
  }

  async fn users_get_by_email(&self, _ctx: Arc<Context>, email: &str) -> Result<User, DBError> {
    let path = "database.users.tokens_create".to_string();

    let row: Result<_, _> = sqlx::query!(
      r#"
       SELECT 
         id, username, email, password_hash, display_name, badges, 
         status_text, status_presence, profile_content, profile_background_id, 
         privileged, suspended_until, created_at, updated_at, verified 
       FROM users
       WHERE email = $1
      "#,
      email,
    )
    .fetch_optional(self.db())
    .await;

    match row {
      Ok(Some(r)) => Ok(User {
        id: r.id,
        username: r.username,
        email: r.email,
        password: r.password_hash,
        display_name: r.display_name,
        badges: r.badges.map(|b| b as u32),
        status_text: r.status_text,
        status_presence: UserStatus::from_optional_string(r.status_presence).map(|s| s.to_i32()),
        profile_content: r.profile_content,
        profile_background_id: r.profile_background_id,
        privileged: r.privileged.unwrap_or_default(),
        suspended_until: r.suspended_until.map(|su| su as u64),
        created_at: r.created_at as u64,
        updated_at: r.updated_at as u64,
        verified: r.verified.unwrap_or_default(),
      }),
      Ok(None) => {
        let msg = "user not found".to_string();
        Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
      }
      Err(err) => {
        let msg = format!("failed to fetch user by email: {}", err);
        Err(DBError { err_type: ErrorType::DatabaseError, msg, path, err: Box::new(err) })
      }
    }
  }

  async fn tokens_create(&self, _ctx: Arc<Context>, token: &Token) -> Result<(), DBError> {
    let path = "database.users.tokens_create".to_string();

    let result: Result<_, _> = sqlx::query(
      "INSERT INTO tokens (id, user_id, token, type, used, created_at, expires_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .bind(&token.token)
    .bind(token.r#type.to_string())
    .bind(token.used)
    .bind(token.created_at)
    .bind(token.expires_at)
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

        let msg = format!("failed to create a token: {}", err);
        Err(DBError { err_type, msg, path, err: Box::new(err) })
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
      Ok(None) => {
        let msg = "user not found".to_string();
        Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
      }
      Err(err) => {
        let msg = format!("failed to fetch user auth data: {}", err);
        Err(DBError { err_type: ErrorType::DatabaseError, msg, path, err: Box::new(err) })
      }
    }
  }

  async fn tokens_get_by_token(&self, _ctx: Arc<Context>, token: &str) -> Result<Token, DBError> {
    let path = "database.users.tokens_get_by_token".to_string();

    let row: Result<_, _> = sqlx::query!(
      r#"
        SELECT id, user_id, token, type, used, created_at, expires_at
        FROM tokens
        WHERE token = $1
      "#,
      token,
    )
    .fetch_optional(self.db())
    .await;

    match row {
      Ok(Some(r)) => Ok(Token {
        id: r.id,
        user_id: r.user_id,
        token: r.token,
        r#type: TokenType::from_string(&r.r#type),
        used: r.used,
        created_at: r.created_at,
        expires_at: r.expires_at,
      }),
      Ok(None) => {
        let msg = "token not found".to_string();
        Err(DBError { err_type: ErrorType::NotFound, msg, path, ..Default::default() })
      }
      Err(err) => {
        let msg = format!("failed to fetch token: {}", err);
        Err(DBError { err_type: ErrorType::DatabaseError, msg, path, err: Box::new(err) })
      }
    }
  }

  async fn tokens_mark_as_used(&self, _ctx: Arc<Context>, token_id: &str) -> Result<(), DBError> {
    let path = "database.users.tokens_mark_as_used".to_string();

    let result: Result<_, _> = sqlx::query("UPDATE tokens SET used = true WHERE id = $1")
      .bind(token_id)
      .execute(self.db())
      .await;

    match result {
      Ok(_) => Ok(()),
      Err(err) => {
        let msg = format!("failed to mark token as used: {}", err);
        Err(DBError { err_type: ErrorType::DatabaseError, msg, path, err: Box::new(err) })
      }
    }
  }
}
