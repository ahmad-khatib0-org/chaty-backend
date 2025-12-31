use std::fmt;
use std::io::Error;

use chaty_config::Settings;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use redpanda::{RedpandaBuilder, RedpandaProducer};

/// Broker connection and topic configuration for Redpanda
pub struct BrokerConfig {
  pub producer: RedpandaProducer,
  pub email_confirmation_topic: String,
  pub password_reset_topic: String,
  pub user_created_topic: String,
}

impl fmt::Debug for BrokerConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("BrokerConfig")
      .field("email_confirmation_topic", &self.email_confirmation_topic)
      .field("password_reset_topic", &self.password_reset_topic)
      .field("user_created_topic", &self.user_created_topic)
      .finish()
  }
}

impl Clone for BrokerConfig {
  fn clone(&self) -> Self {
    Self {
      producer: self.producer.clone(),
      email_confirmation_topic: self.email_confirmation_topic.clone(),
      password_reset_topic: self.password_reset_topic.clone(),
      user_created_topic: self.user_created_topic.clone(),
    }
  }
}

impl BrokerConfig {
  /// Initialize Redpanda broker connection and verify topics
  pub async fn new(settings: &Settings) -> Result<Self, BoxedErr> {
    let ie = |msg: &str, err: BoxedErr| {
      let path = "api.server.broker.new".into();
      InternalError { err_type: ErrorType::InternalError, temp: false, err, msg: msg.into(), path }
    };

    let broker_addrs = settings.kafka.brokers.join(",");
    let mut builder = RedpandaBuilder::new();
    builder.set_bootstrap_servers(&broker_addrs);

    let producer = builder.build_producer().map_err(|err| {
      ie(
        "failed to initialize redpanda producer",
        Box::new(Error::new(std::io::ErrorKind::Other, format!("{:?}", err))),
      )
    })?;

    // Note: Topic creation/verification would typically be handled by:
    // - rpk CLI tool
    // - Redpanda admin API
    // For now, we assume topics are already created

    Ok(BrokerConfig {
      producer,
      email_confirmation_topic: settings.topics.email_confirmation.clone(),
      password_reset_topic: settings.topics.password_reset.clone(),
      user_created_topic: settings.topics.user_created.clone(),
    })
  }
}
