use async_trait::async_trait;
use chaty_proto::{Server, ServerMember};

use crate::{
  models::helpers::ops::shared::server_members_get_ranking, HelpersRepository, ScyllaDb,
};

#[async_trait]
impl HelpersRepository for ScyllaDb {
  fn server_members_get_ranking(&self, member: &ServerMember, server: &Server) -> i64 {
    server_members_get_ranking(member, server)
  }
}
