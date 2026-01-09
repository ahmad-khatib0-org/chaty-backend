/// User record from CDC changefeed
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserRecord {
  pub id: String,
  pub username: String,
  #[serde(default)]
  pub display_name: Option<String>,
  #[serde(default)]
  pub profile_background_id: Option<String>,
}

/// Represents a user CDC message from CockroachDB changefeed
/// Format: {after?, before?, resolved?}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserCDCMessage {
  pub after: Option<UserRecord>,
  pub before: Option<UserRecord>,
  pub resolved: Option<String>,
}
