pub mod users_create;

use std::sync::Arc;

use chaty_config::Settings;
use chaty_result::errors::BoxedErr;
use tracing::info;

use crate::email::EmailService;

pub struct WorkerApiArgs {
  pub config: Arc<Settings>,
  pub email_service: Arc<dyn EmailService>,
}

pub struct WorkerApi {
  pub config: Arc<Settings>,
  pub email_service: Arc<dyn EmailService>,
}

impl WorkerApi {
  /// Initialize worker
  pub async fn new(args: WorkerApiArgs) -> Result<Self, BoxedErr> {
    Ok(WorkerApi { config: args.config, email_service: args.email_service })
  }

  /// Start all message consumers
  pub async fn start(&self) -> Result<(), BoxedErr> {
    info!("Starting workers...");

    // Start email confirmation consumer
    self.start_email_confirmation_consumer().await?;

    Ok(())
  }
}
