mod hydra;
mod metrics;
mod redis;
mod response;
mod router;
mod routes;
mod token;
mod user_cache;

pub mod otel;

use std::net::SocketAddr;
use std::sync::Arc;

use chaty_config::Settings;
use chaty_database::DatabaseSql;
use chaty_proto::envoy_service::auth::v3::authorization_server::AuthorizationServer;
use chaty_result::errors::BoxedErr;
use chaty_result::middleware_context;
use deadpool_redis::Pool as RedisPool;
use hydra::DefaultHydraClient;
use redis::DefaultRedisClient;
use reqwest::Client;
use tonic::service::InterceptorLayer;
use tonic::transport::Server as TonicServer;
use tower::ServiceBuilder;
use tracing::info;

pub struct ControllerArgs {
  pub config: Arc<Settings>,
  pub redis_con: Arc<RedisPool>,
  pub sql_db: Arc<DatabaseSql>,
  pub metrics: metrics::MetricsCollector,
}

pub struct Controller {
  pub config: Arc<Settings>,
  pub hydra: DefaultHydraClient,
  pub redis: DefaultRedisClient,
  pub redis_con: Arc<RedisPool>,
  pub(super) store: Arc<DatabaseSql>,
  pub metrics: metrics::MetricsCollector,
  cached_config: CachedConfig,
}

#[derive(Debug, Default)]
struct CachedConfig {
  pub available_languages: Vec<String>,
  pub default_language: String,
}

impl Controller {
  pub async fn new(ca: ControllerArgs) -> Self {
    let available_languages = ca.config.available_languages.clone();
    let default_language = ca.config.default_language.clone();

    let cached_config = CachedConfig { available_languages, default_language };

    let hydra = DefaultHydraClient {
      hydra_url: ca.config.oauth.admin_url.clone(),
      http: Arc::new(Client::new()),
      client_id: ca.config.oauth.client_id.clone(),
      client_secret: ca.config.oauth.client_secret.clone(),
    };

    let redis = DefaultRedisClient { redis: ca.redis_con.clone(), metrics: ca.metrics.clone() };

    Self {
      config: ca.config,
      hydra,
      redis,
      redis_con: ca.redis_con,
      store: ca.sql_db,
      metrics: ca.metrics,
      cached_config,
    }
  }

  pub async fn run(self) -> Result<(), BoxedErr> {
    let url = &self.config.hosts.auth.clone();

    let layer = ServiceBuilder::new().layer(InterceptorLayer::new(middleware_context)).into_inner();
    info!("the auth server is listening on: {}", url);
    TonicServer::builder()
      .layer(layer)
      .add_service(AuthorizationServer::new(self))
      .serve((url.parse::<SocketAddr>()).unwrap())
      .await?;

    Ok(())
  }
}
