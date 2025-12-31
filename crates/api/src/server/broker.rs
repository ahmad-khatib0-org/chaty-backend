use std::io::{Error, ErrorKind};

use chaty_config::Settings;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;
use tracing::info;

/// Broker connection and topic configuration for Redpanda
pub struct BrokerApi {
  pub producer: FutureProducer,
  pub password_reset_topic: String,
  pub user_created_topic: String,
  pub email_confirmation_topic: String,
  pub email_confirmation_dlq_topic: String,
}

impl std::fmt::Debug for BrokerApi {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BrokerConfig")
      .field("email_confirmation_topic", &self.email_confirmation_topic)
      .field("password_reset_topic", &self.password_reset_topic)
      .field("user_created_topic", &self.user_created_topic)
      .field("email_confirmation_dlq_topic", &self.email_confirmation_dlq_topic)
      .finish()
  }
}

impl BrokerApi {
  /// Initialize Redpanda broker connection
  pub async fn new(settings: &Settings) -> Result<Self, BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| {
      let path = "api.server.broker.new".into();
      InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
    };

    let broker_addrs = settings.kafka.brokers.join(",");

    let producer: FutureProducer = ClientConfig::new()
      .set("bootstrap.servers", &broker_addrs)
      .set("acks", "all")
      .set("retries", "3")
      .set("message.send.max.retries", "3")
      .set("retry.backoff.ms", "100")
      .create()
      .map_err(|err| {
        ie(
          "failed to initialize redpanda producer",
          Box::new(Error::new(std::io::ErrorKind::Other, format!("{:?}", err))),
        )
      })?;

    info!("Redpanda broker initialized with brokers: {}", broker_addrs);

    Ok(BrokerApi {
      producer,
      email_confirmation_topic: settings.topics.email_confirmation.clone(),
      password_reset_topic: settings.topics.password_reset.clone(),
      user_created_topic: settings.topics.user_created.clone(),
      email_confirmation_dlq_topic: settings.topics.email_confirmation_dlq.clone(),
    })
  }

  /// Publish email confirmation message
  pub async fn publish_email_confirmation(
    &self,
    message: &serde_json::Value,
  ) -> Result<(), BoxedErr> {
    let payload = serde_json::to_string(message).map_err(|e| Box::new(e) as BoxedErr)?;
    let key = message.get("user_id").and_then(|v| v.as_str()).unwrap_or("unknown");

    self
      .producer
      .send(
        FutureRecord::to(&self.email_confirmation_topic).payload(&payload).key(key),
        Timeout::After(Duration::from_secs(30)),
      )
      .await
      .map_err(|(err, _)| {
        Box::new(Error::new(std::io::ErrorKind::Other, format!("Kafka error: {}", err))) as BoxedErr
      })?;

    info!("Published email confirmation message to topic");
    Ok(())
  }

  /// Publish to DLQ (Dead Letter Queue)
  pub async fn publish_email_confirmation_dlq(
    &self,
    message: &serde_json::Value,
  ) -> Result<(), BoxedErr> {
    let payload = serde_json::to_string(message).map_err(|e| Box::new(e) as BoxedErr)?;
    let key = message.get("user_id").and_then(|v| v.as_str()).unwrap_or("unknown");

    self
      .producer
      .send(
        FutureRecord::to(&self.email_confirmation_dlq_topic).payload(&payload).key(key),
        Timeout::After(Duration::from_secs(30)),
      )
      .await
      .map_err(|(err, _)| {
        Box::new(Error::new(ErrorKind::Other, format!("Kafka error: {}", err))) as BoxedErr
      })?;

    info!("Published message to DLQ topic");
    Ok(())
  }
}
