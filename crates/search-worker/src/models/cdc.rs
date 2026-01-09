use serde_json::Value;

/// Represents a user CDC message from CockroachDB
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserCDCMessage {
  /// The key (always user_id)
  pub key: Vec<Value>,
  /// The after state of the document
  pub after: Option<UserDocument>,
  /// The before state of the document (for deletes)
  pub before: Option<UserDocument>,
  /// Resolved timestamp (null for regular events)
  pub resolved: Option<String>,
}

/// User document fields for indexing
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserDocument {
  pub id: String,
  pub username: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub display_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub profile_background_id: Option<String>,
}
