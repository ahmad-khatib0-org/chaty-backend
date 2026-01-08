use std::time::Duration;
use std::{collections::HashMap, env, fs, path::Path};

use cached::proc_macro::cached;
use futures_locks::RwLock;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::warn;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "sentry")]
pub use sentry::{capture_error, capture_message, Level};

#[cfg(feature = "anyhow")]
pub use sentry_anyhow::capture_anyhow;

#[derive(Deserialize, Debug, Clone)]
pub struct Database {
  pub scylladb: String,
  pub db_name: String,
  pub postgres: String,
  pub dragonfly: String,
}

impl Default for Database {
  fn default() -> Self {
    Self {
      scylladb: "localhost:9042".to_string(),
      db_name: "chaty".to_string(),
      postgres: "postgresql://chaty@localhost:26257/chaty?sslmode=disable".to_string(),
      dragonfly: "redis://0.0.0.0:6379".to_string(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Kafka {
  pub brokers: Vec<String>,
  pub username: Option<String>,
  pub password: Option<String>,
  pub sasl_mechanism: Option<String>,
  pub security_protocol: Option<String>,
}

impl Default for Kafka {
  fn default() -> Self {
    Self {
      brokers: vec!["localhost:19092".to_string()],
      username: None,
      password: None,
      sasl_mechanism: None,
      security_protocol: None,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Topics {
  pub password_reset: String,
  pub password_reset_dlq: String,
  pub user_created: String,
  pub email_confirmation: String,
  pub email_confirmation_dlq: String,
}

impl Default for Topics {
  fn default() -> Self {
    Self {
      user_created: "api.users.user_created".to_string(),
      password_reset: "api.users.password_reset".to_string(),
      password_reset_dlq: "api.users.password_reset_dlq".to_string(),
      email_confirmation: "api.users.email_confirmation".to_string(),
      email_confirmation_dlq: "api.users.email_confirmation_dlq".to_string(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct OAuth {
  pub public_url: String,
  pub admin_url: String,
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uri: String,
  pub scopes: Vec<String>,
  pub token_endpoint: String,
  pub auth_endpoint: String,
  pub userinfo_endpoint: String,
  pub confirmation_url: String,
  pub reset_password_url: String,
}

impl Default for OAuth {
  fn default() -> Self {
    Self {
      public_url: "http://localhost:4444".to_string(),
      admin_url: "http://localhost:4445".to_string(),
      client_id: String::new(),
      client_secret: String::new(),
      redirect_uri: "http://localhost:3000/api/auth/callback".to_string(),
      token_endpoint: "http://localhost:4444/oauth2/token".to_string(),
      auth_endpoint: "http://localhost:4444/oauth2/auth".to_string(),
      userinfo_endpoint: "http://localhost:4444/userinfo".to_string(),
      scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
      confirmation_url: "http://localhost:3000/api/auth/email-confirmation".to_string(),
      reset_password_url: "http://localhost:3000/api/auth/reset-password".to_string(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hosts {
  pub app: String,
  pub api: String,
  pub ws: String,
  pub files: String,
  pub gifs: String,
  pub auth: String,
  pub livekit: HashMap<String, String>,
  pub otel_collector: String,
  pub api_metrics: String,
  pub auth_metrics: String,
}

impl Default for Hosts {
  fn default() -> Self {
    Self {
      app: "http://localhost:3000".to_string(),
      api: "http://localhost:3001".to_string(),
      ws: "ws://localhost:3002".to_string(),
      files: "http://localhost:3003".to_string(),
      gifs: "http://localhost:3004".to_string(),
      auth: "0.0.0.0:50051".to_string(),
      livekit: HashMap::new(),
      otel_collector: "http://0.0.0.0:4317".to_string(),
      api_metrics: "0.0.0.0:8888".to_string(),
      auth_metrics: "0.0.0.0:8889".to_string(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiRegistration {
  pub invite_only: bool,
}

impl Default for ApiRegistration {
  fn default() -> Self {
    Self { invite_only: false }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSmtp {
  pub host: String,
  pub username: String,
  pub password: String,
  pub from_address: String,
  pub reply_to: Option<String>,
  pub port: Option<i32>,
  pub use_tls: Option<bool>,
  pub use_starttls: Option<bool>,
}

impl Default for ApiSmtp {
  fn default() -> Self {
    Self {
      host: "localhost".to_string(),
      username: "smtp".to_string(),
      password: "smtp".to_string(),
      from_address: "noreply@chaty.local".to_string(),
      reply_to: None,
      port: Some(1025),
      use_tls: Some(false),
      use_starttls: Some(false),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiEmailSendGrid {
  pub api_key: String,
  pub from_address: String,
  pub reply_to: Option<String>,
}

impl Default for ApiEmailSendGrid {
  fn default() -> Self {
    Self { api_key: String::new(), from_address: String::new(), reply_to: None }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiEmail {
  #[serde(default = "default_email_provider")]
  pub provider: String,
  pub smtp: ApiSmtp,
  pub sendgrid: ApiEmailSendGrid,
}

fn default_email_provider() -> String {
  "smtp".to_string()
}

impl Default for ApiEmail {
  fn default() -> Self {
    Self {
      provider: "smtp".to_string(),
      smtp: ApiSmtp::default(),
      sendgrid: ApiEmailSendGrid::default(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PushVapid {
  pub queue: String,
  pub private_key: String,
  pub public_key: String,
}

impl Default for PushVapid {
  fn default() -> Self {
    Self {
      queue: "notifications.outbound.vapid".to_string(),
      private_key: String::new(),
      public_key: String::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PushFcm {
  pub queue: String,
  pub key_type: String,
  pub project_id: String,
  pub private_key_id: String,
  pub private_key: String,
  pub client_email: String,
  pub client_id: String,
  pub auth_uri: String,
  pub token_uri: String,
  pub auth_provider_x509_cert_url: String,
  pub client_x509_cert_url: String,
}

impl Default for PushFcm {
  fn default() -> Self {
    Self {
      queue: "notifications.outbound.fcm".to_string(),
      key_type: String::new(),
      project_id: String::new(),
      private_key_id: String::new(),
      private_key: String::new(),
      client_email: String::new(),
      client_id: String::new(),
      auth_uri: String::new(),
      token_uri: String::new(),
      auth_provider_x509_cert_url: String::new(),
      client_x509_cert_url: String::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PushApn {
  pub queue: String,
  pub sandbox: bool,
  pub pkcs8: String,
  pub key_id: String,
  pub team_id: String,
}

impl Default for PushApn {
  fn default() -> Self {
    Self {
      queue: "notifications.outbound.apn".to_string(),
      sandbox: true,
      pkcs8: String::new(),
      key_id: String::new(),
      team_id: String::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSecurityCaptcha {
  pub hcaptcha_key: String,
  pub hcaptcha_sitekey: String,
}

impl Default for ApiSecurityCaptcha {
  fn default() -> Self {
    Self { hcaptcha_key: String::new(), hcaptcha_sitekey: String::new() }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiSecurity {
  pub captcha: ApiSecurityCaptcha,
  pub trust_cloudflare: bool,
  pub easypwned: String,
  pub tenor_key: String,
}

impl Default for ApiSecurity {
  fn default() -> Self {
    Self {
      captcha: ApiSecurityCaptcha::default(),
      trust_cloudflare: false,
      easypwned: String::new(),
      tenor_key: String::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiWorkers {
  pub max_concurrent_connections: usize,
}

impl Default for ApiWorkers {
  fn default() -> Self {
    Self { max_concurrent_connections: 50 }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiLiveKit {
  pub call_ring_duration: usize,
  pub nodes: HashMap<String, LiveKitNode>,
}

impl Default for ApiLiveKit {
  fn default() -> Self {
    Self { call_ring_duration: 30, nodes: HashMap::new() }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiveKitNode {
  pub url: String,
  pub lat: f64,
  pub lon: f64,
  pub key: String,
  pub secret: String,

  // whether to hide the node in the nodes list
  #[serde(default)]
  pub private: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiUsers {
  pub early_adopter_cutoff: Option<u64>,
}

impl Default for ApiUsers {
  fn default() -> Self {
    Self { early_adopter_cutoff: None }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Api {
  pub registration: ApiRegistration,
  pub email: ApiEmail,
  pub security: ApiSecurity,
  pub workers: ApiWorkers,
  pub livekit: ApiLiveKit,
  pub users: ApiUsers,
}

impl Default for Api {
  fn default() -> Self {
    Self {
      registration: ApiRegistration::default(),
      email: ApiEmail::default(),
      security: ApiSecurity::default(),
      workers: ApiWorkers::default(),
      livekit: ApiLiveKit::default(),
      users: ApiUsers::default(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pushd {
  pub production: bool,
  pub exchange: String,
  pub mass_mention_chunk_size: usize,

  // Queues
  pub message_queue: String,
  pub mass_mention_queue: String,
  pub dm_call_queue: String,
  pub fr_accepted_queue: String,
  pub fr_received_queue: String,
  pub generic_queue: String,
  pub ack_queue: String,

  pub vapid: PushVapid,
  pub fcm: PushFcm,
  pub apn: PushApn,
}

impl Default for Pushd {
  fn default() -> Self {
    Self {
      production: false,
      exchange: "chaty.notifications".to_string(),
      mass_mention_chunk_size: 200,
      message_queue: "notifications.origin.message".to_string(),
      mass_mention_queue: "notifications.origin.mass_mention".to_string(),
      dm_call_queue: "notifications.ingest.dm_call".to_string(),
      fr_accepted_queue: "notifications.ingest.fr_accepted".to_string(),
      fr_received_queue: "notifications.ingest.fr_received".to_string(),
      generic_queue: "notifications.ingest.generic".to_string(),
      ack_queue: "notifications.process.ack".to_string(),
      vapid: PushVapid::default(),
      fcm: PushFcm::default(),
      apn: PushApn::default(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FilesLimit {
  pub min_file_size: usize,
  pub min_resolution: [usize; 2],
  pub max_mega_pixels: usize,
  pub max_pixel_side: usize,
}

impl Default for FilesLimit {
  fn default() -> Self {
    Self { min_file_size: 1, min_resolution: [1, 1], max_mega_pixels: 40, max_pixel_side: 10000 }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FilesS3 {
  pub endpoint: String,
  pub path_style_buckets: bool,
  pub region: String,
  pub access_key_id: String,
  pub secret_access_key: String,
  pub default_bucket: String,
}

impl Default for FilesS3 {
  fn default() -> Self {
    Self {
      endpoint: "http://localhost:9000".to_string(),
      path_style_buckets: true,
      region: "us-east-1".to_string(),
      access_key_id: "chaty-dev".to_string(),
      secret_access_key: "chaty-dev-password".to_string(),
      default_bucket: "chaty-uploads".to_string(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Files {
  pub encryption_key: String,
  pub webp_quality: f32,
  pub blocked_mime_types: Vec<String>,
  pub clamd_host: String,
  pub scan_mime_types: Vec<String>,

  pub limit: FilesLimit,
  pub preview: HashMap<String, [usize; 2]>,
  pub s3: FilesS3,
}

impl Default for Files {
  fn default() -> Self {
    Self {
      encryption_key: String::new(),
      webp_quality: 0.8,
      blocked_mime_types: vec![],
      clamd_host: "localhost:3310".to_string(),
      scan_mime_types: vec![],
      limit: FilesLimit::default(),
      preview: HashMap::new(),
      s3: FilesS3::default(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GlobalLimits {
  pub group_size: usize,
  pub message_embeds: usize,
  pub message_replies: usize,
  pub message_reactions: usize,
  pub server_emoji: usize,
  pub server_roles: usize,
  pub server_channels: usize,

  pub new_user_hours: usize,

  pub body_limit_size: usize,
}

impl Default for GlobalLimits {
  fn default() -> Self {
    Self {
      group_size: 100,
      message_embeds: 5,
      message_replies: 5,
      message_reactions: 20,
      server_emoji: 100,
      server_roles: 200,
      server_channels: 200,
      new_user_hours: 72,
      body_limit_size: 20000000,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FeaturesLimits {
  pub outgoing_friend_requests: usize,

  pub bots: usize,
  pub message_length: usize,
  pub message_attachments: usize,
  pub servers: usize,
  pub voice_quality: u32,
  pub video: bool,
  pub video_resolution: [u32; 2],
  pub video_aspect_ratio: [f32; 2],

  pub file_upload_size_limit: HashMap<String, usize>,
}

impl Default for FeaturesLimits {
  fn default() -> Self {
    Self {
      outgoing_friend_requests: 10,
      bots: 5,
      message_length: 2000,
      message_attachments: 5,
      servers: 100,
      voice_quality: 16000,
      video: true,
      video_resolution: [1080, 720],
      video_aspect_ratio: [0.3, 2.5],
      file_upload_size_limit: HashMap::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FeaturesLimitsCollection {
  pub global: GlobalLimits,

  pub new_user: FeaturesLimits,
  pub default: FeaturesLimits,

  #[serde(flatten)]
  pub roles: HashMap<String, FeaturesLimits>,
}

impl Default for FeaturesLimitsCollection {
  fn default() -> Self {
    Self {
      global: GlobalLimits::default(),
      new_user: FeaturesLimits {
        outgoing_friend_requests: 5,
        bots: 2,
        message_length: 2000,
        message_attachments: 5,
        servers: 50,
        voice_quality: 16000,
        video: true,
        video_resolution: [1080, 720],
        video_aspect_ratio: [0.3, 2.5],
        file_upload_size_limit: HashMap::new(),
      },
      default: FeaturesLimits::default(),
      roles: HashMap::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FeaturesAdvanced {
  #[serde(default)]
  pub process_message_delay_limit: u16,
}

impl Default for FeaturesAdvanced {
  fn default() -> Self {
    Self { process_message_delay_limit: 5 }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Features {
  pub limits: FeaturesLimitsCollection,
  pub webhooks_enabled: bool,
  pub mass_mentions_send_notifications: bool,
  pub mass_mentions_enabled: bool,

  #[serde(default)]
  pub advanced: FeaturesAdvanced,
}

impl Default for Features {
  fn default() -> Self {
    Self {
      limits: FeaturesLimitsCollection::default(),
      webhooks_enabled: true,
      mass_mentions_send_notifications: false,
      mass_mentions_enabled: true,
      advanced: FeaturesAdvanced::default(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Search {
  pub host: String,
  pub api_key: String,
  pub index_usernames: String,
  pub request_timeout_seconds: u64,
}

impl Default for Search {
  fn default() -> Self {
    Self {
      host: "http://localhost:7700".to_string(),
      api_key: String::new(),
      index_usernames: "users".to_string(),
      request_timeout_seconds: 30,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Sentry {
  pub api: String,
  pub ws: String,
  pub voice_ingress: String,
  pub files: String,
  pub proxy: String,
  pub pushd: String,
  pub crond: String,
  pub gifs: String,
}

impl Default for Sentry {
  fn default() -> Self {
    Self {
      api: String::new(),
      ws: String::new(),
      voice_ingress: String::new(),
      files: String::new(),
      proxy: String::new(),
      pushd: String::new(),
      crond: String::new(),
      gifs: String::new(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
  pub database: Database,
  pub kafka: Kafka,
  pub topics: Topics,
  pub oauth: OAuth,
  pub hosts: Hosts,
  pub api: Api,
  pub pushd: Pushd,
  pub files: Files,
  pub features: Features,
  pub search: Search,
  pub sentry: Sentry,
  pub production: bool,
  pub available_languages: Vec<String>,
  pub default_language: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      database: Database::default(),
      kafka: Kafka::default(),
      topics: Topics::default(),
      oauth: OAuth::default(),
      hosts: Hosts::default(),
      api: Api::default(),
      pushd: Pushd::default(),
      files: Files::default(),
      features: Features::default(),
      search: Search::default(),
      sentry: Sentry::default(),
      production: false,
      default_language: "en".to_string(),
      available_languages: vec!["en".to_string()],
    }
  }
}

impl Pushd {
  fn get_routing_key(&self, key: String) -> String {
    match self.production {
      true => key + "-prd",
      false => key + "-tst",
    }
  }

  pub fn get_ack_routing_key(&self) -> String {
    self.get_routing_key(self.ack_queue.clone())
  }

  pub fn get_message_routing_key(&self) -> String {
    self.get_routing_key(self.message_queue.clone())
  }

  pub fn get_mass_mention_routing_key(&self) -> String {
    self.get_routing_key(self.mass_mention_queue.clone())
  }

  pub fn get_dm_call_routing_key(&self) -> String {
    self.get_routing_key(self.dm_call_queue.clone())
  }

  pub fn get_fr_accepted_routing_key(&self) -> String {
    self.get_routing_key(self.fr_accepted_queue.clone())
  }

  pub fn get_fr_received_routing_key(&self) -> String {
    self.get_routing_key(self.fr_received_queue.clone())
  }

  pub fn get_generic_routing_key(&self) -> String {
    self.get_routing_key(self.generic_queue.clone())
  }
}

impl Settings {
  pub fn preflight_checks(&self) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize tracing subscriber for structured logging
    let subscriber =
      tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer());

    let _ = tracing::subscriber::set_default(subscriber);

    if self.api.email.provider == "smtp" && self.api.email.smtp.host.is_empty() {
      warn!("No SMTP settings specified! Remember to configure email.");
    }

    if self.api.email.provider == "sendgrid" && self.api.email.sendgrid.api_key.is_empty() {
      warn!("No SendGrid API key specified! Remember to configure email.");
    }

    if self.api.security.captcha.hcaptcha_key.is_empty() {
      warn!("No Captcha key specified! Remember to add hCaptcha key.");
    }
  }
}

/// Configure logging and common Rust variables
#[cfg(feature = "sentry")]
pub async fn setup_logging(release: &'static str, dsn: String) -> Option<sentry::ClientInitGuard> {
  if dsn.is_empty() {
    None
  } else {
    Some(sentry::init((
      dsn,
      sentry::ClientOptions { release: Some(release.into()), ..Default::default() },
    )))
  }
}

#[cfg(feature = "sentry")]
#[macro_export]
macro_rules! configure {
  ($application: ident) => {
    let config = $crate::config().await;
    let _sentry = $crate::setup_logging(
      concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")),
      config.sentry.$application,
    )
    .await;
  };
}

/// Configuration builder
static CONFIG_BUILDER: Lazy<RwLock<Settings>> = Lazy::new(|| {
  RwLock::new({
    let mut env_mode = env::var("ENV").unwrap_or("local".to_string());
    if env_mode != "dev" && env_mode != "local" && env_mode != "prod" {
      env_mode = "local".to_string();
    }

    let path = format!("chaty.{}.yaml", env_mode);
    let mut settings = Settings::default();

    if Path::new(&path).exists() {
      let settings_str = fs::read_to_string(path).expect("Should read config file");
      settings = serde_yaml::from_str(&settings_str).expect("Should deserialize config file");
    } else {
      println!("warn: the config with path {} , is not exists !", path);
    }
    settings
  })
});

pub async fn read() -> Settings {
  CONFIG_BUILDER.read().await.clone()
}

#[cached(time = 300)]
pub async fn config() -> Settings {
  let mut config = read().await;

  // auto-detect production nodes
  if config.hosts.api.contains("https") {
    config.production = true;
  }

  config
}
