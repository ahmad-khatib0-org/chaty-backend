use chaty_result::errors::BoxedErr;

use chaty_auth::server::Server;

#[tokio::main]
async fn main() -> Result<(), BoxedErr> {
  let server = Server::new().await;

  match server {
    Ok(mut srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
