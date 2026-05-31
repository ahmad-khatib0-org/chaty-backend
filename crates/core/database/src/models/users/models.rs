use chaty_config::config;
use chaty_proto::{ApiUser, User, UserRelationship, UserRelationshipStatus, UserStatus};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::EnumHelpers;

pub enum UserBadges {
  /// Chaty Developer
  Developer = 1,
  /// Helped translate Chaty
  Translator = 2,
  /// Monetarily supported Chaty
  Supporter = 4,
  /// Responsibly disclosed a security issue
  ResponsibleDisclosure = 8,
  /// Chaty Founder
  Founder = 16,
  /// Platform moderator
  PlatformModeration = 32,
  /// Active monetary supporter
  ActiveSupporter = 64,
  /// 🦊🦝
  Paw = 128,
  /// Joined as one of the first 1000 users in 2021
  EarlyAdopter = 256,
  /// Amogus
  ReservedRelevantJokeBadge1 = 512,
  /// Low resolution troll face
  ReservedRelevantJokeBadge2 = 1024,
}

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

  pub fn to_i32(&self) -> i32 {
    match self {
      TokenType::EmailVerification => 0,
      TokenType::PasswordReset => 1,
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

pub trait UserApiExt {
  /// Convert user object into ApiUser model assuming mutual connection
  ///
  /// Relations will never be included, i.e. when we process ourselves
  fn into_user_api<'a>(
    self,
    perspective: Option<&'a User>,
    is_online: bool,
  ) -> BoxFuture<'a, ApiUser>;

  /// Gets the user's badges along with calculating any dynamic badges
  fn get_badges(&self) -> BoxFuture<'_, u32>;
}

impl UserApiExt for User {
  fn into_user_api<'a>(
    self,
    perspective: Option<&'a User>,
    is_online: bool,
  ) -> BoxFuture<'a, ApiUser> {
    Box::pin(async move {
      let (relationship, can_see_profile) = if self.bot.is_some() {
        (relationship_status_to_string(UserRelationshipStatus::None), true)
      } else if let Some(perspective) = perspective {
        if perspective.id == self.id {
          (relationship_status_to_string(UserRelationshipStatus::User), true)
        } else {
          let relationship = perspective
            .relations
            .iter()
            .find(|relationship: &&UserRelationship| relationship.id == self.id)
            .map(|rel| rel.status())
            .unwrap_or_default();

          let can_see_profile = relationship != UserRelationshipStatus::BlockedOther;
          (relationship_status_to_string(relationship), can_see_profile)
        }
      } else {
        (relationship_status_to_string(UserRelationshipStatus::None), true)
      };

      let badges = self.get_badges().await; // Now this works!

      ApiUser {
        online: Some(
          can_see_profile && is_online && self.status_presence() != UserStatus::Invisible,
        ),
        id: self.id,
        email: self.email,
        username: self.username,
        display_name: self.display_name,
        verified: self.verified,
        avatar: self.avatar,
        relations: vec![],
        badges: Some(badges as i32),
        status_text: self.status_text,
        profile_background_id: self.profile_background_id,
        profile_content: self.profile_content,
        suspended_until: self.suspended_until,
        status_presence: self.status_presence,
        privileged: self.privileged,
        bot: self.bot,
        relationship: relationship.to_string(),
        created_at: self.created_at,
        updated_at: self.updated_at,
      }
    })
  }

  fn get_badges(&self) -> BoxFuture<'_, u32> {
    Box::pin(async move {
      let config = config().await;
      let badges = self.badges.unwrap_or_default() as u32;

      if let Some(cutoff) = config.api.users.early_adopter_cutoff {
        if Ulid::from_string(&self.id).unwrap().timestamp_ms() < cutoff {
          return badges + UserBadges::EarlyAdopter as u32;
        };
      };

      badges
    })
  }
}

pub fn relationship_status_to_string(status: UserRelationshipStatus) -> &'static str {
  match status {
    UserRelationshipStatus::None => "none",
    UserRelationshipStatus::User => "user",
    UserRelationshipStatus::Friend => "friend",
    UserRelationshipStatus::Outgoing => "outgoing",
    UserRelationshipStatus::Incoming => "incoming",
    UserRelationshipStatus::Blocked => "blocked",
    UserRelationshipStatus::BlockedOther => "blocked_other",
  }
}
