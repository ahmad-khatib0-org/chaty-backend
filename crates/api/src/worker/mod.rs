pub mod users_create;
pub mod users_forgot_password;

use std::sync::Arc;

use chaty_config::Settings;
use chaty_result::errors::BoxedErr;
use tokio::spawn;
use tracing::info;

use crate::email::EmailService;

pub struct WorkerApiArgs {
  pub config: Arc<Settings>,
  pub email_service: Arc<dyn EmailService>,
}

#[derive(Clone)]
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

    let email_consumer = self.clone();
    spawn(async move {
      if let Err(e) = email_consumer.start_email_confirmation_consumer().await {
        tracing::error!("Email confirmation consumer error: {:?}", e);
      }
    });

    let password_reset_consumer = self.clone();
    spawn(async move {
      if let Err(e) = password_reset_consumer.start_password_reset_consumer().await {
        tracing::error!("Password reset consumer error: {:?}", e);
      }
    });

    // Give consumers time to initialize before returning
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok(())
  }
}
