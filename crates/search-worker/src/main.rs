use chaty_result::errors::BoxedErr;
use chaty_search_worker::server::SearchWorkerServer;

#[tokio::main]
async fn main() -> Result<(), BoxedErr> {
  let server = SearchWorkerServer::new().await;

  match server {
    Ok(srv) => return srv.run().await,
    Err(e) => Err(e),
  }
}
