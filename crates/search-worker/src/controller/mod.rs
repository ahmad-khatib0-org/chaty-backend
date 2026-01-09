mod commit_coordinator;
mod consumer_shutdown;
mod shutdown;
mod task_poller;
mod usernames_consumer;
mod usernames_message_processor;
mod usernames_task_processor;

use std::{
  collections::HashMap,
  io::{Error, ErrorKind},
  sync::{atomic::AtomicBool, Arc},
  time::Duration,
};

use chaty_config::Settings;
use chaty_database::DatabaseSql;
use chaty_result::errors::{BoxedErr, ErrorType, InternalError};
use rdkafka::{
  consumer::{Consumer, StreamConsumer},
  producer::FutureProducer,
  ClientConfig,
};
use reqwest::Client;
use serde_json::json;
use tokio::{
  sync::{watch, Mutex, Notify, Semaphore},
  task::JoinSet,
};
use tracing::info;

use crate::server::observability::MetricsCollector;

/// Key type for offset tracking: (topic, partition)
type Key = (String, i32);

pub struct SearchWorkerControllerArgs {
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) metrics: Arc<MetricsCollector>,
}

#[derive(Clone)]
pub(crate) struct SearchWorkerController {
  pub(super) sql_db: Arc<DatabaseSql>,
  pub(super) config: Arc<Settings>,
  pub(super) metrics: Arc<MetricsCollector>,
  pub(super) http_client: Arc<Client>,
  // Support multiple consumers: HashMap<name, consumer>
  pub(crate) consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer>>>>,
  // Topic to consumer mapping: HashMap<topic, consumer_name>
  pub(crate) topic_to_consumer: Arc<Mutex<HashMap<String, String>>>,
  pub(crate) producer: Arc<FutureProducer>,
  // Shutdown coordination
  pub(crate) shutdown_notify: Arc<Notify>,
  pub(crate) tx_metrics_shutdown: watch::Sender<()>,
  // Task acceptance flag: when false, stop spawning new tasks and drain
  pub(crate) task_accepting: Arc<AtomicBool>,
  // Concurrency control
  pub(crate) semaphore: Arc<Semaphore>,
  pub(crate) join_set: Arc<Mutex<JoinSet<()>>>,
  // Offset tracking for commit coordination: (topic, partition) -> highest_offset_seen
  pub(crate) highest_offset: Arc<Mutex<HashMap<Key, i64>>>,
}

impl SearchWorkerController {
  pub fn new(args: SearchWorkerControllerArgs) -> SearchWorkerController {
    let http_client = Client::builder()
      .timeout(Duration::from_secs(10))
      .connect_timeout(Duration::from_secs(3))
      .pool_idle_timeout(Duration::from_secs(90))
      .pool_max_idle_per_host(10)
      .build()
      .expect("Failed to create reqwest client");

    let config = &args.config;

    // Create usernames consumer
    let usernames_consumer: Arc<StreamConsumer> = Arc::new(
      ClientConfig::new()
        .set("bootstrap.servers", config.kafka.brokers.join(","))
        .set("group.id", "search-worker-usernames")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create kafka consumer"),
    );

    usernames_consumer
      .subscribe(&[&config.topics.search_users_changes])
      .expect("Failed to subscribe to search topic");

    // Initialize consumers HashMap with usernames consumer
    let mut consumers_map = HashMap::new();
    consumers_map.insert("usernames".to_string(), usernames_consumer);

    // Initialize topic to consumer mapping
    let mut topic_to_consumer_map = HashMap::new();
    topic_to_consumer_map
      .insert(config.topics.search_users_changes.clone(), "usernames".to_string());

    let producer: Arc<FutureProducer> = Arc::new(
      ClientConfig::new()
        .set("bootstrap.servers", config.kafka.brokers.join(","))
        .create()
        .expect("Failed to create kafka producer"),
    );

    let shutdown_notify = Arc::new(Notify::new());
    let (tx_metrics_shutdown, _rx_metrics_shutdown) = watch::channel(());
    let task_accepting = Arc::new(AtomicBool::new(true));
    let semaphore = Arc::new(Semaphore::new(100)); // Max 100 concurrent tasks
    let join_set = Arc::new(Mutex::new(JoinSet::new()));
    let highest_offset: Arc<Mutex<HashMap<Key, i64>>> = Arc::new(Mutex::new(HashMap::new()));

    SearchWorkerController {
      sql_db: args.sql_db,
      config: args.config,
      metrics: args.metrics,
      http_client: Arc::new(http_client),
      consumers: Arc::new(Mutex::new(consumers_map)),
      topic_to_consumer: Arc::new(Mutex::new(topic_to_consumer_map)),
      producer,
      shutdown_notify,
      tx_metrics_shutdown,
      task_accepting,
      semaphore,
      join_set,
      highest_offset,
    }
  }

  /// Run the search worker controller
  /// This function blocks and coordinates all search worker operations
  pub async fn run(self) -> Result<(), BoxedErr> {
    info!("Starting search worker controller");

    // Setup Meilisearch indexes
    self.indexes_setup().await?;

    // Start shutdown listener
    self.shutdown_listener();

    // Start periodic commit task
    self.periodic_commit();

    // Start consuming usernames topic
    self.usernames_consumer().await;

    // Gracefully shutdown consumer
    self.consumer_shutdown().await;

    info!("Search worker controller shutdown complete");
    Ok(())
  }

  /// Setup Meilisearch indexes before consuming messages
  async fn indexes_setup(&self) -> Result<(), BoxedErr> {
    info!("Setting up Meilisearch indexes");

    let index_names =
      vec![&self.config.search.index_usernames, &self.config.search.index_usernames_dlq];

    for idx_name in index_names.iter() {
      self.ensure_index_exists(&idx_name).await?;
    }

    info!("Meilisearch indexes setup complete");
    Ok(())
  }

  /// Ensure Meilisearch index exists
  async fn ensure_index_exists(&self, index_name: &str) -> Result<(), BoxedErr> {
    let ie = |err: BoxedErr, msg: &str| {
      let path = "search-worker.controller.ensure_index_exists".into();
      let err_type = ErrorType::InternalError;
      InternalError { err_type, temp: false, err, msg: msg.into(), path }
    };

    let url = format!("{}/indexes", self.config.search.host.clone());
    let payload = json!({ "uid": index_name, "primaryKey": "id" });

    let mut req = self.http_client.post(&url).json(&payload);

    let api_key = self.config.search.api_key.clone();
    if !api_key.is_empty() {
      req = req.bearer_auth(api_key);
    }

    let resp = req
      .send()
      .await
      .map_err(|e| Box::new(ie(Box::new(e), "failed to send create index request")) as BoxedErr)?;

    let status = resp.status();
    // 202 = accepted (created)
    if !status.is_success() {
      let txt = resp.text().await.unwrap_or_default();
      let err = Box::new(Error::new(ErrorKind::Other, "meilisearch_error"));
      let msg = &format!("create index failed: status={}, body={}", status, txt);
      return Err(Box::new(ie(err, msg)));
    }

    Ok(())
  }
}
