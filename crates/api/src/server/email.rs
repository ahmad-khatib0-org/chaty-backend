use std::sync::Arc;

use chaty_config::Settings;
use chaty_result::errors::BoxedErr;
use tracing::info;

/// Email service trait for abstraction
#[async_trait::async_trait]
pub trait EmailService: Send + Sync {
  async fn send(
    &self,
    to: &str,
    subject: &str,
    html_body: &str,
    text_body: &str,
  ) -> Result<(), BoxedErr>;
}

/// SMTP Email Service
pub struct SmtpEmailService {
  host: String,
  port: u16,
  username: String,
  password: String,
  from_address: String,
  use_tls: bool,
  use_starttls: bool,
}

impl SmtpEmailService {
  pub fn new(config: &chaty_config::ApiSmtp) -> Self {
    SmtpEmailService {
      host: config.host.clone(),
      port: config.port.unwrap_or(25) as u16,
      username: config.username.clone(),
      password: config.password.clone(),
      from_address: config.from_address.clone(),
      use_tls: config.use_tls.unwrap_or(false),
      use_starttls: config.use_starttls.unwrap_or(false),
    }
  }
}

#[async_trait::async_trait]
impl EmailService for SmtpEmailService {
  async fn send(
    &self,
    to: &str,
    subject: &str,
    html_body: &str,
    text_body: &str,
  ) -> Result<(), BoxedErr> {
    info!("Sending email via SMTP to: {}", to);

    // TODO: Implement actual SMTP sending using lettre
    // For now, just log the intent
    // use lettre::transport::smtp::SmtpTransport;
    // use lettre::{Message, Transport};
    // use lettre::message::MultiPart;
    //
    // let email = Message::builder()
    //   .from(self.from_address.parse()?)
    //   .to(to.parse()?)
    //   .subject(subject)
    //   .multipart(MultiPart::alternative()
    //     .singlepart(lettre::message::SinglePart::plain(text_body.to_string()))
    //     .singlepart(lettre::message::SinglePart::html(html_body.to_string()))
    //   )?;
    //
    // let transport = SmtpTransport::builder_dangerous(&self.host)
    //   .port(self.port)
    //   .build();
    //
    // transport.send(&email)?;

    info!("Email would be sent via SMTP: {} bytes HTML", html_body.len());
    Ok(())
  }
}

/// SendGrid Email Service
pub struct SendGridEmailService {
  api_key: String,
  from_address: String,
}

impl SendGridEmailService {
  pub fn new(config: &chaty_config::ApiEmailSendGrid) -> Self {
    SendGridEmailService {
      api_key: config.api_key.clone(),
      from_address: config.from_address.clone(),
    }
  }
}

#[async_trait::async_trait]
impl EmailService for SendGridEmailService {
  async fn send(
    &self,
    to: &str,
    subject: &str,
    html_body: &str,
    text_body: &str,
  ) -> Result<(), BoxedErr> {
    info!("Sending email via SendGrid to: {}", to);

    let payload = serde_json::json!({
      "personalizations": [
        {
          "to": [
            {
              "email": to
            }
          ]
        }
      ],
      "from": {
        "email": self.from_address
      },
      "subject": subject,
      "content": [
        {
          "type": "text/plain",
          "value": text_body
        },
        {
          "type": "text/html",
          "value": html_body
        }
      ]
    });

    // TODO: Implement actual SendGrid API call
    // use reqwest::Client;
    //
    // let client = Client::new();
    // let response = client
    //   .post("https://api.sendgrid.com/v3/mail/send")
    //   .header("Authorization", format!("Bearer {}", self.api_key))
    //   .json(&payload)
    //   .send()
    //   .await?;
    //
    // if !response.status().is_success() {
    //   return Err(Box::new(std::io::Error::new(
    //     std::io::ErrorKind::Other,
    //     format!("SendGrid error: {}", response.text().await?),
    //   )) as BoxedErr);
    // }

    info!("Email would be sent via SendGrid to: {} with subject: {}", to, subject);
    Ok(())
  }
}

/// Email Service Factory
pub fn create_email_service(config: &Settings) -> Result<Arc<dyn EmailService>, BoxedErr> {
  match config.api.email.provider.as_str() {
    "smtp" => {
      info!("Using SMTP email service");
      Ok(Arc::new(SmtpEmailService::new(&config.api.email.smtp)))
    }
    "sendgrid" => {
      info!("Using SendGrid email service");
      if config.api.email.sendgrid.api_key.is_empty() {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::InvalidInput,
          "SendGrid API key not configured",
        )) as BoxedErr);
      }
      Ok(Arc::new(SendGridEmailService::new(&config.api.email.sendgrid)))
    }
    provider => Err(Box::new(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!("Unknown email provider: {}", provider),
    )) as BoxedErr),
  }
}
