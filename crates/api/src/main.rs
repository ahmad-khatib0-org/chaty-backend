use api::ApiServer;
use chaty_result::BoxedErr;

#[tokio::main]
async fn main() -> Result<(), BoxedErr> {
  let server = ApiServer::new().await;
  match server {
    Ok(mut srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}

