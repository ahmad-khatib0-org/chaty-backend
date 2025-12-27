use std::future::Future;
use std::pin::Pin;

#[cfg(feature = "scylladb")]
use crate::drivers::scylladb::ScyllaDb;
mod scylladb;

mod reference;

use crate::drivers::reference::ReferenceDb;
use chaty_config::config;

/// Database information to use to create a client
pub enum DatabaseInfo {
  /// Auto-detect the database in use
  Auto,
  /// Auto-detect the database in use and create an empty testing database
  Test(String),
  /// Use the mock database
  Reference,
  /// Connect to ScyllaDb
  #[cfg(feature = "scylladb")]
  ScyllaDb { uri: String, keyspace: String },
}

/// Database
#[derive(Debug)]
pub enum Database {
  /// Mock database
  Reference(ReferenceDb),
  /// Scylladb database
  #[cfg(feature = "scylladb")]
  Scylladb(ScyllaDb),
}

// Helper type alias and function defined at module scope
type BoxedFuture = Pin<Box<dyn Future<Output = Result<Database, String>>>>;

fn boxed(f: impl Future<Output = Result<Database, String>> + 'static) -> BoxedFuture {
  Box::pin(f)
}

impl DatabaseInfo {
  /// Create a database client from the given database information
  pub async fn connect(self) -> Result<Database, String> {
    let config = config().await;
    match self {
      DatabaseInfo::Auto => {
        if std::env::var("TEST_DB").is_ok() {
          boxed(DatabaseInfo::Test("chaty_test".to_string()).connect()).await
        } else if !config.database.scylladb.is_empty() {
          #[cfg(feature = "scylladb")]
          {
            boxed(
              DatabaseInfo::ScyllaDb {
                uri: config.database.scylladb,
                keyspace: "chaty".to_string(),
              }
              .connect(),
            )
            .await
          }
          #[cfg(not(feature = "scylladb"))]
          return Err("scylladb not enabled.".to_string());
        } else {
          boxed(DatabaseInfo::Reference.connect()).await
        }
      }
      DatabaseInfo::Test(database_name) => {
        let test_db = std::env::var("TEST_DB")
          .expect("`TEST_DB` environment variable should be set to REFERENCE or SCYLLADB");

        match test_db.as_str() {
          "REFERENCE" => boxed(DatabaseInfo::Reference.connect()).await,
          "SCYLLADB" => {
            #[cfg(feature = "scylladb")]
            {
              boxed(
                DatabaseInfo::ScyllaDb { uri: config.database.scylladb, keyspace: database_name }
                  .connect(),
              )
              .await
            }
            #[cfg(not(feature = "scylladb"))]
            return Err("scylladb not enabled.".to_string());
          }
          _ => unreachable!("must specify REFERENCE or SCYLLADB"),
        }
      }
      #[cfg(feature = "scylladb")]
      DatabaseInfo::ScyllaDb { uri, keyspace } => {
        use scylla::client::session::Session;
        use scylla::client::session_builder::SessionBuilder;

        let session: Session = SessionBuilder::new()
          .known_node(uri)
          .build()
          .await
          .map_err(|e| format!("Failed to connect to ScyllaDB: {}", e))?;

        session
          .use_keyspace(&keyspace, false)
          .await
          .map_err(|e| format!("Failed to use keyspace: {}", e))?;

        Ok(Database::Scylladb(ScyllaDb(session)))
      }
      DatabaseInfo::Reference => Ok(Database::Reference(Default::default())),
    }
  }
}

