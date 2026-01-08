use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Task response from create/update/delete operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
  /// Unique task identifier (e.g., 1, 2, 3)
  pub task_uid: u64,

  /// Index UID the task belongs to
  pub index_uid: String,

  /// Task status: "enqueued", "processing", "succeeded", "failed"
  pub status: TaskStatus,

  /// Task type: "documentAdditionOrUpdate", "documentDeletion", "settingsUpdate", etc.
  #[serde(rename = "type")]
  pub task_type: TaskType,

  /// When the task was enqueued (RFC3339 format)
  pub enqueued_at: String,

  /// Added in v1.1.0: Optional fields that may appear in some responses
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uid: Option<u64>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub canceled_by: Option<u64>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<Value>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<TaskError>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration: Option<String>, // ISO 8601 duration format

  #[serde(skip_serializing_if = "Option::is_none")]
  pub started_at: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub finished_at: Option<String>,
}

/// Detailed task information from GET /tasks/:taskUid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
  /// Unique task identifier
  pub uid: u64,

  /// Index UID the task belongs to
  pub index_uid: String,

  /// Task status
  pub status: TaskStatus,

  /// Task type
  #[serde(rename = "type")]
  pub task_type: TaskType,

  /// Task that canceled this task (if any)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub canceled_by: Option<u64>,

  /// Task-specific details
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<TaskDetails>,

  /// Error information (if task failed)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<TaskError>,

  /// Duration of task processing (ISO 8601 format)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration: Option<String>,

  /// When the task was enqueued
  pub enqueued_at: String,

  /// When the task started processing
  #[serde(skip_serializing_if = "Option::is_none")]
  pub started_at: Option<String>,

  /// When the task finished
  #[serde(skip_serializing_if = "Option::is_none")]
  pub finished_at: Option<String>,
}

/// Task status enum based on MeiliSearch documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
  /// Task is waiting to be processed
  Enqueued,

  /// Task is being processed
  Processing,

  /// Task completed successfully
  Succeeded,

  /// Task failed
  Failed,

  /// Task was canceled by another task
  Canceled,
}

/// Task type enum based on MeiliSearch documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskType {
  /// Add or update documents
  DocumentAdditionOrUpdate,

  /// Delete documents
  DocumentDeletion,

  /// Clear all documents
  DocumentClear,

  /// Update settings
  SettingsUpdate,

  /// Delete index
  IndexDeletion,

  /// Create index
  IndexCreation,

  IndexUpdate,

  /// Dump creation
  DumpCreation,

  /// Task deletion
  TaskDeletion,

  /// Task cancelation
  TaskCancelation,

  /// Snapshot creation
  SnapshotCreation,

  /// Swap indexes
  IndexSwap,
}

/// Task error information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
  /// Human-readable error message
  pub message: String,

  /// Error code
  pub code: String,

  /// Error type
  #[serde(rename = "type")]
  pub error_type: String,

  /// Link to documentation
  #[serde(skip_serializing_if = "Option::is_none")]
  pub link: Option<String>,
}

/// Task details - varies by task type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskDetails {
  /// Details for document addition/update
  DocumentAdditionOrUpdate {
    /// Number of documents received
    received_documents: u64,

    /// Number of documents indexed
    indexed_documents: u64,

    /// Primary key used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_key: Option<String>,
  },

  /// Details for document deletion
  DocumentDeletion {
    /// Number of deleted documents
    deleted_documents: u64,

    /// Original filter (if used)
    #[serde(skip_serializing_if = "Option::is_none")]
    original_filter: Option<String>,
  },

  /// Details for settings update
  SettingsUpdate {
    /// Settings that were updated
    settings: Value,
  },

  /// Details for index creation
  IndexCreation {
    /// Primary key for the index
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_key: Option<String>,
  },

  /// Details for index swap
  IndexSwap {
    /// Indexes that were swapped
    swaps: Vec<IndexSwap>,
  },

  /// Generic details for other task types
  Generic(Value),
}

/// Index swap information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSwap {
  /// Indexes to swap
  pub indexes: Vec<String>,
}

/// Tasks response wrapper (for GET /tasks)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksResponse {
  /// Array of tasks
  pub results: Vec<Task>,

  /// Pagination information
  pub limit: u64,

  /// Current offset
  pub from: Option<u64>,

  /// Next offset for pagination
  pub next: Option<u64>,

  /// Total number of tasks
  pub total: u64,
}

/// Task deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDeletionResponse {
  /// Task UID that was deleted
  pub task_uid: u64,

  /// Index UID
  pub index_uid: String,

  /// Task status
  pub status: TaskStatus,

  /// Task type
  #[serde(rename = "type")]
  pub task_type: TaskType,

  /// When the task was enqueued
  pub enqueued_at: String,
}

/// Task cancellation response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCancelationResponse {
  /// Original task UID that was canceled
  pub original_task_uid: u64,

  /// Cancellation task UID
  pub task_uid: u64,

  /// Index UID
  pub index_uid: String,

  /// Task status
  pub status: TaskStatus,

  /// Task type (should be "taskCancelation")
  #[serde(rename = "type")]
  pub task_type: TaskType,

  /// When the task was enqueued
  pub enqueued_at: String,
}
