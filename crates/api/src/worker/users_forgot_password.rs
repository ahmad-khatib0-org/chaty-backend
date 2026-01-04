use std::{
  collections::HashMap,
  io::{Error, ErrorKind},
};

use chaty_result::{errors::BoxedErr, tr};
use rdkafka::{
  consumer::{Consumer, StreamConsumer},
  ClientConfig, Message,
};
use serde_json::Value;
use tera::Tera;
use tokio_stream::StreamExt;
use tracing::{error, info};

use crate::worker::WorkerApi;

/// Password reset message structure
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PasswordResetMessage {
  pub user_id: String,
  pub email: String,
  pub username: String,
  pub reset_token: String,
  pub language: String,
}

impl WorkerApi {
  /// Start the password reset consumer
  pub async fn start_password_reset_consumer(&self) -> Result<(), BoxedErr> {
    let broker_addrs = self.config.kafka.brokers.join(",");
    let topic = &self.config.topics.password_reset;

    let consumer: StreamConsumer = ClientConfig::new()
      .set("bootstrap.servers", &broker_addrs)
      .set("group.id", "password-reset-group")
      .set("auto.offset.reset", "earliest")
      .set("enable.auto.commit", "true")
      .create()
      .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))?;

    consumer.subscribe(&[topic]).map_err(|e| Box::new(e))?;

    info!("Password reset consumer started for topic: {}", topic);

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
      match result {
        Ok(message) => {
          let payload = match message.payload_view::<str>() {
            Some(Ok(p)) => p,
            Some(Err(e)) => {
              error!("Failed to deserialize payload: {:?}", e);
              continue;
            }
            None => {
              error!("Empty password reset message payload");
              continue;
            }
          };

          match serde_json::from_str::<PasswordResetMessage>(payload) {
            Ok(msg) => {
              if let Err(e) = self.process_password_reset(&msg).await {
                error!("Failed to process password reset for user {}: {:?}", msg.user_id, e);
              }
            }
            Err(e) => {
              error!("Failed to deserialize password reset message: {:?}", e);
              continue;
            }
          }
        }
        Err(e) => {
          error!("Consumer error: {:?}", e);
        }
      }
    }

    Ok(())
  }

  /// Process password reset email
  async fn process_password_reset(&self, msg: &PasswordResetMessage) -> Result<(), BoxedErr> {
    let reset_link = format!("{}?token={}", self.config.oauth.reset_password_url, msg.reset_token);

    let subject = tr::<()>(&msg.language, "email.password_reset.subject", None)
      .unwrap_or_else(|_| "Reset Your Password".to_string());

    let mut tera = Tera::new("crates/api/src/worker/templates/*.html")?;
    let mut context = tera::Context::new();

    let greeting = tr(
      &msg.language,
      "email.password_reset.greeting",
      Some(HashMap::from([("username", Value::String(msg.username.to_string()))])),
    )
    .unwrap_or_else(|_| format!("Hello {},", msg.username));

    let intro = tr::<()>(&msg.language, "email.password_reset.intro", None).unwrap_or_else(|_| {
      "We received a request to reset your password. Click the button below to reset it:"
        .to_string()
    });

    let button_text = tr::<()>(&msg.language, "email.password_reset.button_text", None)
      .unwrap_or_else(|_| "Reset Password".to_string());

    let alt_text = tr::<()>(&msg.language, "email.password_reset.alt_text", None)
      .unwrap_or_else(|_| "Or copy and paste this link into your browser:".to_string());

    let expiry = tr::<()>(&msg.language, "email.password_reset.expiry_notice", None)
      .unwrap_or_else(|_| "This link expires in 24 hours.".to_string());

    let signature = tr::<()>(&msg.language, "email.password_reset.signature", None)
      .unwrap_or_else(|_| "Best regards,<br>The Chaty Team".to_string());

    context.insert("greeting", &greeting);
    context.insert("intro", &intro);
    context.insert("button_text", &button_text);
    context.insert("reset_link", &reset_link);
    context.insert("alt_text", &alt_text);
    context.insert("expiry", &expiry);
    context.insert("signature", &signature);

    let html_body = tera.render("password_reset.html", &context)?;

    // Render text version
    tera = Tera::new("crates/api/src/worker/templates/*.txt")?;
    let text_body = tera.render("password_reset.txt", &context)?;

    self.email_service.send(&msg.email, &subject, &html_body, &text_body).await?;

    info!("Password reset email sent to {} for user {}", msg.email, msg.user_id);

    Ok(())
  }
}
