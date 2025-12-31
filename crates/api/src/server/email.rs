use std::{
  io::{Error, ErrorKind},
  sync::Arc,
  time::Duration,
};

use chaty_config::{ApiEmailSendGrid, ApiSmtp, Settings};
use chaty_result::errors::BoxedErr;
use lettre::{
  message::{MultiPart, SinglePart},
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
    SmtpTransport,
  },
  Message, Transport,
};
use reqwest::Client;
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
  config: ApiSmtp,
}

impl SmtpEmailService {
  pub fn new(config: ApiSmtp) -> Self {
    SmtpEmailService { config }
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

    let from_address = self.config.from_address.parse().map_err(|e| {
      Box::new(Error::new(ErrorKind::InvalidInput, format!("Invalid from address: {}", e)))
    })?;

    let to_address = to.parse().map_err(|e| {
      Box::new(Error::new(ErrorKind::InvalidInput, format!("Invalid recipient address: {}", e)))
    })?;

    let email = Message::builder()
      .from(from_address)
      .to(to_address)
      .subject(subject)
      .multipart(
        MultiPart::alternative()
          .singlepart(SinglePart::plain(text_body.to_string()))
          .singlepart(SinglePart::html(html_body.to_string())),
      )
      .map_err(|e| Box::new(Error::new(ErrorKind::InvalidInput, e)))?;

    let port = self.config.port.unwrap_or(587) as u16; // 587 is standard for STARTTLS

    let mut builder = SmtpTransport::relay(&self.config.host)
      .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))?
      .port(port);

    let use_tls = self.config.use_tls.unwrap_or(false);
    let use_starttls = self.config.use_starttls.unwrap_or(false);

    if use_tls {
      // Implicit TLS (usually port 465)
      builder = builder.tls(Tls::Required(
        TlsParameters::new(self.config.host.clone())
          .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))?,
      ));
    } else if use_starttls {
      // STARTTLS (usually port 587)
      builder = builder.tls(Tls::Required(
        TlsParameters::new(self.config.host.clone())
          .map_err(|e| Box::new(Error::new(ErrorKind::Other, e)))?,
      ));
    } else {
      builder = builder.tls(Tls::None);
    }

    // Handle Credentials
    if !self.config.username.is_empty() {
      let credentials =
        Credentials::new(self.config.username.clone(), self.config.password.clone());
      builder = builder.credentials(credentials);
    }

    let transport = builder.build();

    transport
      .send(&email)
      .map_err(|e| Box::new(Error::new(ErrorKind::Other, format!("SMTP send failed: {}", e))))?;

    info!("Email sent successfully via SMTP to: {}", to);
    Ok(())
  }
}

/// SendGrid Email Service
pub struct SendGridEmailService {
  http_client: Arc<Client>,
  api_key: String,
  from_address: String,
}

impl SendGridEmailService {
  pub fn new(config: &ApiEmailSendGrid) -> Self {
    let http_client = reqwest::Client::builder()
      .timeout(Duration::from_secs(10)) // Don't hang forever
      .connect_timeout(Duration::from_secs(3))
      .pool_idle_timeout(Duration::from_secs(90))
      .pool_max_idle_per_host(10) // Keep connections alive for reuse
      .build()
      .expect("Failed to create reqwest client for SendGrid");

    SendGridEmailService {
      http_client: Arc::new(http_client),
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

    let client = self.http_client.clone();
    let response = client
      .post("https://api.sendgrid.com/v3/mail/send")
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&payload)
      .send()
      .await
      .map_err(|e| {
        Box::new(Error::new(ErrorKind::Other, format!("SendGrid request failed: {}", e)))
      })?;

    if !response.status().is_success() {
      let status = response.status();
      let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(Box::new(Error::new(
        ErrorKind::Other,
        format!("SendGrid error ({}): {}", status, error_body),
      )));
    }

    info!("Email sent successfully via SendGrid to: {}", to);
    Ok(())
  }
}

/// Email Service Factory
pub fn create_email_service(config: &Settings) -> Result<Arc<dyn EmailService>, BoxedErr> {
  match config.api.email.provider.as_str() {
    "smtp" => {
      info!("Using SMTP email service");
      Ok(Arc::new(SmtpEmailService::new(config.api.email.smtp.clone())))
    }
    "sendgrid" => {
      info!("Using SendGrid email service");
      if config.api.email.sendgrid.api_key.is_empty() {
        let err = Error::new(ErrorKind::InvalidInput, "SendGrid API key not configured");
        return Err(Box::new(err));
      }
      Ok(Arc::new(SendGridEmailService::new(&config.api.email.sendgrid)))
    }
    provider => {
      let msg = format!("Unknown email provider: {}", provider);
      Err(Box::new(Error::new(ErrorKind::InvalidInput, msg)))
    }
  }
}
