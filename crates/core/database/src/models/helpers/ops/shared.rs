use std::{str::FromStr, time::Duration};

use chaty_config::{config, FeaturesLimits};
use chaty_proto::{Server, ServerMember};
use ulid::Ulid;

pub fn server_members_get_ranking(member: &ServerMember, server: &Server) -> i64 {
  let mut value = i64::MAX;
  for role in &member.roles {
    if let Some(role) = server.roles.get(role) {
      if role.rank < value {
        value = role.rank;
      }
    }
  }
  value
}

pub(super) async fn users_get_limits(user_id: &str) -> FeaturesLimits {
  let config = config().await;
  if Ulid::from_str(user_id)
    .expect("should be ulid")
    .datetime()
    .elapsed()
    .expect("time should not go backwards")
    <= Duration::from_secs(3600u64 * config.features.limits.global.new_user_hours as u64)
  {
    config.features.limits.new_user
  } else {
    config.features.limits.default
  }
}
