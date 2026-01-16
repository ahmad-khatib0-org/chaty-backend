use chaty_proto::{Server, ServerMember};

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
