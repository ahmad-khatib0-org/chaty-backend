use std::io::{Error, ErrorKind};

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

/// Email confirmation message structure
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EmailConfirmationMessage {
  pub user_id: String,
  pub email: String,
  pub username: String,
  pub confirmation_token: String,
  pub language: String,
}

impl EmailConfirmationMessage {
  pub fn from_json(value: &Value) -> Result<Self, serde_json::Error> {
    serde_json::from_value(value.clone())
  }
}

impl WorkerApi {
  /// Start the email confirmation consumer
  pub async fn start_email_confirmation_consumer(&self) -> Result<(), BoxedErr> {
    let broker_addrs = self.config.kafka.brokers.join(",");
    let topic = &self.config.topics.email_confirmation;

    let consumer: StreamConsumer = ClientConfig::new()
      .set("bootstrap.servers", &broker_addrs)
      .set("group.id", "email-confirmation-group")
      .set("auto.offset.reset", "earliest")
      .set("enable.auto.commit", "true")
      .create()
      .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))?;

    consumer.subscribe(&[topic]).map_err(|e| Box::new(e))?;

    info!("Email confirmation consumer started for topic: {}", topic);

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
              error!("Failed to get message payload");
              continue;
            }
          };

          match serde_json::from_str::<EmailConfirmationMessage>(payload) {
            Ok(email_msg) => {
              match self.process_email_confirmation(email_msg.clone()).await {
                Ok(_) => {
                  info!("Successfully processed email for: {}", email_msg.email);
                }
                Err(e) => {
                  error!("Failed to process email: {:?}", e);
                  // TODO: Publish to DLQ on processing failure
                }
              }
            }
            Err(e) => {
              error!("Failed to deserialize message: {:?}", e);
            }
          }
        }
        Err(e) => {
          error!("Kafka error: {}", e);
        }
      }
    }

    Ok(())
  }

  /// Process email confirmation message
  pub async fn process_email_confirmation(
    &self,
    message: EmailConfirmationMessage,
  ) -> Result<(), BoxedErr> {
    info!("Processing email confirmation for user: {}", message.user_id);
    let lang = &message.language;

    let email_confirmation_subject = tr::<()>(lang, "email.confirmation.subject", None)
      .unwrap_or("Confirm Your Email Address".to_string());
    let email_confirmation_greeting = tr::<()>(lang, "email.confirmation.greeting", None)
      .unwrap_or(format!("Hello {}", message.username).to_string());
    let email_confirmation_intro = tr::<()>(lang, "email.confirmation.intro", None)
      .unwrap_or("Thank you for creating an account with Chaty! To complete your registration, please confirm your email address by clicking the button below:".to_string());
    let email_confirmation_button_text =
      tr::<()>(lang, "email.confirmation.button_text", None).unwrap_or("Confirm Email".to_string());
    let email_confirmation_alt_text = tr::<()>(lang, "email.confirmation.alt_text", None)
      .unwrap_or("Or copy and paste this link into your browser:".to_string());
    let email_confirmation_expiry_notice = tr::<()>(lang, "email.confirmation.expiry_notice", None)
      .unwrap_or("This link expires in 24 hours.".to_string());
    let email_confirmation_not_requested = tr::<()>(lang, "email.confirmation.not_requested", None)
      .unwrap_or("email.confirmation.not_requested".to_string());
    let email_confirmation_signature = tr::<()>(lang, "email.confirmation.signature", None)
      .unwrap_or("Best regards,<br>The Chaty Team".to_string());
    let email_footer_copyright = tr::<()>(lang, "email.footer.copyright", None)
      .unwrap_or("&copy; 2024 Chaty. All rights reserved.".to_string());

    let base = self.config.oauth.confirmation_url.clone();
    // Render templates with user data using Tera
    let confirmation_url = format!("{}?token={}", base, message.confirmation_token);

    let mut context = tera::Context::new();
    context.insert("username", &message.username);
    context.insert("email", &message.email);
    context.insert("confirmation_url", &confirmation_url);
    context.insert("user_id", &message.user_id);
    context.insert("confirmation_token", &message.confirmation_token);

    // Insert translated strings
    context.insert("email_confirmation_subject", &email_confirmation_subject);
    context.insert("email_confirmation_greeting", &email_confirmation_greeting);
    context.insert("email_confirmation_intro", &email_confirmation_intro);
    context.insert("email_confirmation_button_text", &email_confirmation_button_text);
    context.insert("email_confirmation_alt_text", &email_confirmation_alt_text);
    context.insert("email_confirmation_expiry_notice", &email_confirmation_expiry_notice);
    context.insert("email_confirmation_not_requested", &email_confirmation_not_requested);
    context.insert("email_confirmation_signature", &email_confirmation_signature);
    context.insert("email_footer_copyright", &email_footer_copyright);

    let mut tera = Tera::default();
    tera.add_raw_template("html", include_str!("templates/email_confirmation.html"))?;
    tera.add_raw_template("text", include_str!("templates/email_confirmation.txt"))?;

    let html_body = tera.render("html", &context)?;
    let text_body = tera.render("text", &context)?;

    self
      .email_service
      .send(&message.email, &email_confirmation_subject, &html_body, &text_body)
      .await?;

    info!("Email confirmation sent to: {}", message.email);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_email_confirmation_message_deserialization() {
    let json = json!({
      "user_id": "123",
      "email": "test@example.com",
      "username": "testuser",
      "confirmation_token": "confirm_abc123",
      "language": "en"
    });

    let msg = EmailConfirmationMessage::from_json(&json).unwrap();
    assert_eq!(msg.user_id, "123");
    assert_eq!(msg.email, "test@example.com");
    assert_eq!(msg.username, "testuser");
    assert_eq!(msg.confirmation_token, "confirm_abc123");
    assert_eq!(msg.language, "en");
  }

  #[test]
  fn test_email_confirmation_message_default_language() {
    let json = json!({
      "user_id": "123",
      "email": "test@example.com",
      "username": "testuser",
      "confirmation_token": "confirm_abc123"
    });

    let msg = EmailConfirmationMessage::from_json(&json).unwrap();
    assert_eq!(msg.language, "en");
  }
}
