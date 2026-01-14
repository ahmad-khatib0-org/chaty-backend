use std::future::Future;
use std::pin::Pin;

pub use self::reference_no_sql::*;
pub use self::reference_sql::*;

mod reference_no_sql;
mod reference_sql;

#[cfg(feature = "scylladb")]
pub use self::scylladb::*;
mod scylladb;

#[cfg(feature = "postgres")]
pub use self::postgres::*;
mod postgres;

use chaty_config::config;

/// Database information to use to create a client
pub enum DatabaseInfoNoSql {
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
pub enum DatabaseNoSql {
  /// Mock database
  Reference(ReferenceNoSqlDb),
  /// Scylladb database
  #[cfg(feature = "scylladb")]
  Scylladb(ScyllaDb),
}

// Generic helper type alias and function
type BoxedFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>>>>;

fn boxed<T>(f: impl Future<Output = Result<T, String>> + 'static) -> BoxedFuture<T> {
  Box::pin(f)
}

impl DatabaseInfoNoSql {
  /// Create a database client from the given database information
  pub async fn connect(self) -> Result<DatabaseNoSql, String> {
    let config = config().await;
    match self {
      DatabaseInfoNoSql::Auto => {
        if std::env::var("TEST_DB_NO_SQL").is_ok() {
          boxed(DatabaseInfoNoSql::Test("chaty_test".to_string()).connect()).await
        } else if !config.database.scylladb.is_empty() {
          #[cfg(feature = "scylladb")]
          {
            boxed(
              DatabaseInfoNoSql::ScyllaDb {
                uri: config.database.scylladb,
                keyspace: "chaty".to_string(),
              }
              .connect(),
            )
            .await
          }
        } else {
          boxed(DatabaseInfoNoSql::Reference.connect()).await
        }
      }
      DatabaseInfoNoSql::Test(database_name) => {
        let test_db = std::env::var("TEST_DB_NO_SQL")
          .expect("`TEST_DB_NO_SQL` environment variable should be set to REFERENCE or SCYLLADB");

        match test_db.as_str() {
          "REFERENCE" => boxed(DatabaseInfoNoSql::Reference.connect()).await,
          "SCYLLADB" => {
            #[cfg(feature = "scylladb")]
            {
              boxed(
                DatabaseInfoNoSql::ScyllaDb {
                  uri: config.database.scylladb,
                  keyspace: database_name,
                }
                .connect(),
              )
              .await
            }
          }
          _ => unreachable!("must specify REFERENCE or SCYLLADB"),
        }
      }
      #[cfg(feature = "scylladb")]
      DatabaseInfoNoSql::ScyllaDb { uri, keyspace } => {
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

        let insert_channel = session
          .prepare(
            r#" 
              INSERT INTO channels (
                id, channel_type, "group", created_at, updated_at
              ) 
              VALUES (?, ?, ?, ?, ?)
            "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare insert_channel statment: {}", e))?;

        let insert_channel_by_user = session
          .prepare(
            r#"
              INSERT INTO channels_by_user (
                user_id, channel_id, channel_type, "group", created_at, updated_at
              ) 
              VALUES (?, ?, ?, ?, ?, ?)
            "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare insert_channel_by_user statment: {}", e))?;

        let groups_list_first_page = session
          .prepare(
            r#"
                SELECT channel_id, "group", created_at 
                FROM channels_by_user 
                WHERE user_id = ? 
                ORDER BY channel_id DESC 
                LIMIT ?
            "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare groups_list_first_page statment: {}", e))?;

        // With last_id: Gets groups older than the last one received
        let groups_list_next_page = session
          .prepare(
            r#"
                SELECT channel_id, "group", created_at 
                FROM channels_by_user 
                WHERE user_id = ? AND channel_id < ? 
                ORDER BY channel_id DESC 
                LIMIT ?
            "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare groups_list_next_page statment: {}", e))?;

        let get_server_ids_by_user_id = session
          .prepare(r#"SELECT server_id FROM server_members_by_user WHERE user_id = ?"#)
          .await
          .map_err(|e| format!("Failed to prepare get_server_ids_by_user_id  statment: {}", e))?;

        let insert_channel_by_recipient = session
          .prepare(
            r#"
            INSERT INTO channels_by_recipient(
                recipient_user_id, channel_id, channel_type, created_at
            ) VALUES (?, ?, ?, ?)
           "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare get_server_ids_by_user_id  statment: {}", e))?;

        let get_server_member_by_id = session
          .prepare(
            r#"
                SELECT 
                  user_id, username, avatar, nickname, joined_at,
                  roles, timeout, can_publish, can_receive 
                FROM server_members 
                WHERE server_id = ? AND user_id = ?
           "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare get_server_ids_by_user_id  statment: {}", e))?;

        let get_server_by_id = session
          .prepare(
            r#"
              SELECT 
                 id, owner_id, name, description, 
                 default_permissions, icon, banner, flags, 
                 nsfw, analytics, discoverable, roles, 
                 categories, system_messages, stats, 
                 channels, created_at, updated_at
              FROM servers 
              WHERE id = ?
           "#,
          )
          .await
          .map_err(|e| format!("Failed to prepare get_server_by_id statment: {}", e))?;

        Ok(DatabaseNoSql::Scylladb(ScyllaDb {
          db: session,
          prepared: Prepared {
            servers: PreparedServers { get_server_by_id },
            channels: PreparedChannels {
              insert_channel,
              insert_channel_by_user,
              insert_channel_by_recipient,
              groups_list_first_page,
              groups_list_next_page,
            },
            server_members: PreparedServerMembers {
              get_server_ids_by_user_id,
              get_server_member_by_id,
            },
          },
        }))
      }
      DatabaseInfoNoSql::Reference => Ok(DatabaseNoSql::Reference(Default::default())),
    }
  }
}

pub enum DatabaseInfoSql {
  /// Auto-detect the database in use
  Auto,
  /// Auto-detect the database in use and create an empty testing database
  Test(String),
  /// Use the mock database
  Reference,
  /// Connect to Postgres
  #[cfg(feature = "postgres")]
  Postgres { dsn: String },
}

/// Database
#[derive(Debug)]
pub enum DatabaseSql {
  /// Mock database
  Reference(ReferenceSqlDb),
  /// Postgres database
  #[cfg(feature = "postgres")]
  Postgres(PostgresDb),
}

impl DatabaseInfoSql {
  /// Create a database client from the given database information
  pub async fn connect(self) -> Result<DatabaseSql, String> {
    let config = config().await;
    match self {
      DatabaseInfoSql::Auto => {
        if std::env::var("TEST_DB_SQL").is_ok() {
          boxed(DatabaseInfoSql::Test("chaty_test".to_string()).connect()).await
        } else if !config.database.postgres.is_empty() {
          #[cfg(feature = "postgres")]
          {
            boxed(DatabaseInfoSql::Postgres { dsn: config.database.postgres }.connect()).await
          }
        } else {
          boxed(DatabaseInfoSql::Reference.connect()).await
        }
      }
      DatabaseInfoSql::Test(database_name) => {
        let test_db = std::env::var("TEST_DB_SQL")
          .expect("`TEST_DB_SQL` environment variable should be set to REFERENCE or POSTGRES");

        match test_db.as_str() {
          "REFERENCE" => boxed(DatabaseInfoSql::Reference.connect()).await,
          "POSTGRES" => {
            #[cfg(feature = "postgres")]
            {
              boxed(DatabaseInfoSql::Postgres { dsn: database_name }.connect()).await
            }
          }
          _ => unreachable!("must specify REFERENCE or POSTGRES"),
        }
      }
      #[cfg(feature = "postgres")]
      DatabaseInfoSql::Postgres { dsn } => {
        use std::time::Duration;

        use sqlx::postgres::PgPoolOptions;

        let pool = PgPoolOptions::new()
          .max_connections(10)
          .min_connections(2)
          .max_lifetime(Duration::from_millis(600000))
          .idle_timeout(Duration::from_millis(120000))
          .connect(&dsn)
          .await
          .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

        sqlx::query("SELECT 1")
          .execute(&pool)
          .await
          .map_err(|e| format!("Failed to verify PostgreSQL connection: {}", e))?;

        Ok(DatabaseSql::Postgres(PostgresDb(pool)))
      }
      DatabaseInfoSql::Reference => Ok(DatabaseSql::Reference(Default::default())),
    }
  }
}
