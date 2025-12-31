use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chaty_proto::{User, UsersCreateRequest};
use chaty_result::{context::Context, errors::AppError};
use serde_json::{json, Value};
use tonic::Code;
use validator::ValidateEmail;
use regex::Regex;
use once_cell::sync::Lazy;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use ulid::Ulid;

/// Regex for valid usernames
/// Allows letters, digits, underscores, dots, and hyphens
/// Blocks zero-width spaces and lookalike characters
static RE_USERNAME: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[a-zA-Z0-9._-]{3,60}$").unwrap()
});

pub fn users_create_validate(
  ctx: Arc<Context>,
  path: &str,
  req: &UsersCreateRequest,
) -> Result<(), AppError> {
  let ae = |id: &str| {
    return AppError::new(ctx.clone(), path, id, None, "", Code::InvalidArgument.into(), None);
  };

  // Validate email
  if req.email.trim().is_empty() {
    return Err(ae("users.email.required"));
  }
  if !req.email.validate_email() {
    return Err(ae("users.email.invalid"));
  }
  if req.email.len() > 255 {
    return Err(ae("users.email.too_long"));
  }

  // Validate username
  if req.username.trim().is_empty() {
    return Err(ae("users.username.required"));
  }
  if !RE_USERNAME.is_match(&req.username) {
    return Err(ae("users.username.invalid"));
  }
  if req.username.len() > 64 {
    return Err(ae("users.username.too_long"));
  }

  // Validate password
  if req.password.trim().is_empty() {
    return Err(ae("users.password.required"));
  }
  if req.password.len() < 8 {
    return Err(ae("users.password.too_short"));
  }
  if !req.password.chars().any(|c| c.is_uppercase()) {
    return Err(ae("users.password.requires_uppercase"));
  }
  if !req.password.chars().any(|c| c.is_lowercase()) {
    return Err(ae("users.password.requires_lowercase"));
  }
  if !req.password.chars().any(|c| c.is_numeric()) {
    return Err(ae("users.password.requires_number"));
  }
  if !req.password.chars().any(|c| !c.is_alphanumeric()) {
    return Err(ae("users.password.requires_symbol"));
  }

  Ok(())
}

/// Pre-save function to populate User from request
/// Generates ULID for id, hashes password, and sets timestamps
pub async fn users_create_pre_save(
  ctx: Arc<Context>,
  path: &str,
  req: &UsersCreateRequest,
) -> Result<User, AppError> {
  let ie = |id: &str| {
    return AppError::new(ctx.clone(), path, id, None, "", Code::Internal.into(), None);
  };

  // Generate ULID for user ID
  let user_id = Ulid::new().to_string();

  // Hash password with Argon2
  let salt = SaltString::generate(rand::thread_rng());
  let password_hash = Argon2::default()
    .hash_password(req.password.as_bytes(), &salt)
    .map_err(|_| ie("users.password.hash_failed"))?
    .to_string();

  // Get current timestamp in milliseconds
  let now_millis = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as u64;

  Ok(User {
    id: user_id,
    username: req.username.clone(),
    email: req.email.clone(),
    password: password_hash,
    display_name: None,
    badges: None,
    status_text: None,
    status_presence: None,
    profile_content: None,
    profile_background_id: None,
    privileged: false,
    suspended_until: None,
    created_at: now_millis,
    updated_at: now_millis,
    verified: false,
  })
}

// create an auditable request to be saved
pub fn users_create_auditable(user: &UsersCreateRequest) -> Value {
  json!({ "username": user.username, "email": user.email })
}
