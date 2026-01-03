use chaty_proto::UserStatus;
use serde::{Deserialize, Serialize};

use crate::EnumHelpers;

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

  pub fn from_string(s: &str) -> Self {
    match s {
      "email_confirmation" => TokenType::EmailVerification,
      "password_reset" => TokenType::PasswordReset,
      _ => TokenType::EmailVerification, // Default to EmailVerification
    }
  }
}

impl EnumHelpers for UserStatus {
  fn to_str(&self) -> &'static str {
    match self {
      UserStatus::Online => "online",
      UserStatus::Idle => "idle",
      UserStatus::Focus => "focus",
      UserStatus::Busy => "busy",
      UserStatus::Invisible => "invisible",
    }
  }

  fn from_optional_string(s: Option<String>) -> Option<Self> {
    match s.unwrap_or_default().to_lowercase().as_str() {
      "online" => Some(UserStatus::Online),
      "idle" => Some(UserStatus::Idle),
      "focus" => Some(UserStatus::Focus),
      "busy" => Some(UserStatus::Busy),
      "invisible" => Some(UserStatus::Invisible),
      _ => None,
    }
  }

  fn to_i32(&self) -> i32 {
    match self {
      UserStatus::Online => 0,
      UserStatus::Idle => 1,
      UserStatus::Focus => 2,
      UserStatus::Busy => 3,
      UserStatus::Invisible => 4,
    }
  }
}
