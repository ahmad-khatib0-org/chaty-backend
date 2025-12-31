use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct CachedUserData {
  pub is_oauth: bool,
  pub roles: String,
  pub props: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Token {
  pub id: String,        // VARCHAR(26)
  pub user_id: String,   // VARCHAR NOT NULL
  pub token: String,     // VARCHAR(256) NOT NULL
  pub r#type: TokenType, // VARCHAR(64) NOT NULL
  pub used: bool,        // BOOLEAN NOT NULL DEFAULT FALSE
  pub created_at: i64,   // BIGINT NOT NULL
  pub expires_at: i64,   // BIGINT NOT NULL
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum TokenType {
  EmailVerification,
  PasswordReset,
}

impl TokenType {
  pub fn to_string(&self) -> &str {
    match self {
      TokenType::EmailVerification => "email_confirmation",
      TokenType::PasswordReset => "password_reset",
    }
  }
}
