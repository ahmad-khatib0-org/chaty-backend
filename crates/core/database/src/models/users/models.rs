use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct CachedUserData {
  pub is_oauth: bool,
  pub roles: String,
  /// like: theme:light,mobile_notification:true
  pub props: String,
}
