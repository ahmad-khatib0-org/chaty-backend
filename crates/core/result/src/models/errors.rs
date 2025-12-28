use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use chaty_proto::AppError as AppErrorProto;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tonic::Code;

use crate::{context::Context, tr, TranslateFunc};

pub type BoxedErr = Box<dyn Error + Sync + Send>;
pub type OptionalErr = Option<BoxedErr>;
pub type OptionalParams = Option<HashMap<String, Value>>;

pub const ERROR_ID_CANCELED: &str = "error.canceled";
pub const ERROR_ID_UNKNOWN: &str = "error.unknown";
pub const ERROR_ID_INVALID_ARGUMENT: &str = "error.invalid_argument";
pub const ERROR_ID_DEADLINE_EXCEEDED: &str = "error.deadline_exceeded";
pub const ERROR_ID_NOT_FOUND: &str = "error.not_found";
pub const ERROR_ID_ALREADY_EXISTS: &str = "error.already_exists";
pub const ERROR_ID_PERMISSION_DENIED: &str = "error.permission_denied";
pub const ERROR_ID_RESOURCE_EXHAUSTED: &str = "error.resource_exhausted";
pub const ERROR_ID_FAILED_PRECONDITION: &str = "error.failed_precondition";
pub const ERROR_ID_ABORTED: &str = "error.aborted";
pub const ERROR_ID_OUT_OF_RANGE: &str = "error.out_of_range";
pub const ERROR_ID_UNIMPLEMENTED: &str = "error.unimplemented";
pub const ERROR_ID_INTERNAL: &str = "error.internal";
pub const ERROR_ID_UNAVAILABLE: &str = "error.unavailable";
pub const ERROR_ID_DATA_LOSS: &str = "error.data_loss";
pub const ERROR_ID_UNAUTHENTICATED: &str = "error.unauthenticated";

#[derive(Debug)]
pub struct DBError {
  pub err_type: ErrorType,
  pub err: Box<dyn Error + Send + Sync>,
  pub msg: String,
  pub path: String,
}

impl Default for DBError {
  fn default() -> Self {
    Self {
      err_type: ErrorType::DatabaseError,
      err: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Database error")),
      msg: String::new(),
      path: String::new(),
    }
  }
}

impl fmt::Display for DBError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut parts = Vec::new();

    if !self.path.is_empty() {
      parts.push(format!("path: {}", self.path));
    }
    parts.push(format!("err_type: {}", self.err_type));
    if !self.msg.is_empty() {
      parts.push(format!("msg: {}", self.msg));
    }
    parts.push(format!("err: {}", self.err));

    write!(f, "{}", parts.join(", "))
  }
}

impl Error for DBError {}

impl DBError {
  pub fn new(
    path: impl Into<String>,
    err: Box<dyn Error + Send + Sync>,
    err_type: ErrorType,
    msg: impl Into<String>,
  ) -> Self {
    Self { err_type, err, msg: msg.into(), path: path.into() }
  }
}

#[derive(Debug)]
pub struct SimpleError {
  pub message: String,
  pub _type: ErrorType,
  pub err: BoxedErr,
}

impl fmt::Display for SimpleError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}: {}", self._type, self.message)
  }
}

impl Error for SimpleError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(self.err.as_ref())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorError {
  pub id: String,
  pub params: Option<HashMap<String, Value>>,
}

#[derive(Debug, Default)]
pub struct AppErrorErrors {
  pub err: OptionalErr,
  pub errors: Option<HashMap<String, String>>,
  pub errors_internal: Option<HashMap<String, AppErrorError>>,
}

#[derive(Debug)]
pub struct AppError {
  pub ctx: Arc<Context>,
  pub id: String,
  pub path: String,
  pub message: String,
  pub detailes: String,
  pub status_code: i32,
  pub tr_params: OptionalParams,
  pub skip_translation: bool,
  pub errors: Option<AppErrorErrors>,
}

impl AppError {
  pub fn new(
    ctx: Arc<Context>,
    path: impl Into<String>,
    id: impl Into<String>,
    id_params: OptionalParams,
    details: impl Into<String>,
    status_code: i32,
    errors: Option<AppErrorErrors>,
  ) -> Self {
    let errors = errors.unwrap_or_default();

    let mut err = Self {
      ctx,
      id: id.into(),
      path: path.into(),
      message: "".to_string(),
      detailes: details.into(),
      status_code,
      tr_params: id_params,
      skip_translation: false,
      errors: Some(errors),
    };

    let boxed_tr = Box::new(|lang: &str, id: &str, params: &HashMap<String, serde_json::Value>| {
      let params_option = if params.is_empty() { None } else { Some(params.clone()) };
      tr(lang, id, params_option).map_err(|e| Box::new(e) as Box<dyn Error>)
    });

    err.translate(Some(boxed_tr));
    err
  }

  pub fn error_string(&self) -> String {
    let mut s = String::new();

    if !self.path.is_empty() {
      s.push_str(&self.path);
      s.push_str(": ");
    }

    if !self.message.is_empty() {
      s.push_str(&self.message);
    }

    if !self.detailes.is_empty() {
      s.push_str(&format!(", {}", self.detailes));
    }

    if let Some(err) = self.errors.as_ref().and_then(|e| e.err.as_ref()) {
      s.push_str(&format!(", {}", err));
    }

    s
  }

  pub fn translate(&mut self, tf: Option<TranslateFunc>) {
    if self.skip_translation {
      return;
    }

    if let Some(tf) = tf {
      let empty = HashMap::new();
      let params = self.tr_params.as_ref().unwrap_or(&empty);
      if let Ok(translated) = tf(&self.ctx.accept_language, &self.id, params) {
        self.message = translated;
        return;
      }
    }
    self.message = self.id.clone();
  }

  pub fn unwrap(&self) -> Option<&(dyn Error + Send + Sync)> {
    self.errors.as_ref().and_then(|e| e.err.as_deref())
  }

  pub fn wrap(mut self, err: Box<dyn Error + Send + Sync>) -> Self {
    if let Some(errors) = &mut self.errors {
      errors.err = Some(err);
    } else {
      self.errors = Some(AppErrorErrors { err: Some(err), ..Default::default() });
    }
    self
  }

  pub fn wipe_detailed(&mut self) {
    if let Some(errors) = &mut self.errors {
      errors.err = None;
    }
    self.detailes.clear();
  }

  pub fn default() -> Self {
    Self {
      ctx: Arc::new(Context::default()),
      path: String::new(),
      id: String::new(),
      message: String::new(),
      detailes: String::new(),
      status_code: Code::Ok as i32,
      tr_params: None,
      skip_translation: false,
      errors: None,
    }
  }

  /// Convert to proto-generated struct
  pub fn to_proto(&self) -> AppErrorProto {
    let mut errors: HashMap<String, String> = HashMap::new();

    if let Some(app_errors) = self.errors.as_ref() {
      if let Some(errors_internal) = &app_errors.errors_internal {
        for (key, value) in errors_internal {
          let result =
            tr(&self.ctx.accept_language, &value.id, value.params.clone()).unwrap_or_default();
          errors.insert(key.to_string(), result);
        }
      }
    }

    AppErrorProto {
      id: self.id.clone(),
      location: self.path.clone(),
      message: self.message.clone(),
      detailed_error: self.detailes.clone(),
      status_code: self.status_code as u32,
      skip_translation: Some(self.skip_translation),
      errors,
    }
  }
}

/// Convert from proto-generated struct
pub fn app_error_from_proto_app_error(ctx: Arc<Context>, ae: &AppErrorProto) -> AppError {
  let mut errors_internal = HashMap::new();
  for (key, value) in &ae.errors {
    errors_internal.insert(key.clone(), AppErrorError { id: value.clone(), params: None });
  }

  AppError {
    ctx,
    id: ae.id.clone(),
    path: ae.location.clone(),
    message: ae.message.clone(),
    detailes: ae.detailed_error.clone(),
    status_code: ae.status_code as i32,
    tr_params: None,
    skip_translation: ae.skip_translation.unwrap_or_default(),
    errors: Some(AppErrorErrors {
      err: None,
      errors: None,
      errors_internal: Some(errors_internal),
    }),
  }
}

// Implement std::fmt::Display for error formatting
impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.error_string())
  }
}

impl Error for AppError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    self.errors.as_ref().and_then(|e| e.err.as_ref()).map(|e| e.as_ref() as &(dyn Error + 'static))
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ErrorType {
  // General errors
  LabelMe,
  AlreadyOnboarded,
  InvalidOperation,
  InvalidCredentials,
  InvalidProperty,
  InvalidSession,
  InvalidFlagValue,
  NotAuthenticated,
  DuplicateNonce,
  NotFound,
  NoEffect,
  NoRows,
  NotNullViolation,
  NoEmbedData,
  EmptyMessage,
  PayloadTooLarge,

  // Validation errors
  FailedValidation { error: String },
  InvalidUsername,
  InvalidRole,
  InvalidNumber,
  Base64Invalid,
  RegexInvalid,

  // Rate limiting & limits
  DiscriminatorChangeRatelimited,
  TooManyAttachments { max: usize },
  TooManyEmbeds { max: usize },
  TooManyReplies { max: usize },
  TooManyChannels { max: usize },
  TooManyServers { max: usize },
  TooManyEmoji { max: usize },
  TooManyRoles { max: usize },
  GroupTooLarge { max: usize },
  ReachedMaximumBots,
  FileTooLarge { max: usize },
  FileTooSmall,

  // Existence & uniqueness
  UsernameTaken,
  AlreadyFriends,
  AlreadySentRequest,
  AlreadyInGroup,
  AlreadyInServer,
  AlreadyPinned,
  AlreadyConnected,
  UniqueViolation,
  ResourceExists,

  // User relations
  UnknownUser,
  Blocked,
  BlockedByOther,
  NotFriends,
  NotInGroup,
  CannotRemoveYourself,
  CannotTimeoutYourself,
  CannotReportYourself,
  TooManyPendingFriendRequests { max: usize },

  // Channel & message errors
  UnknownChannel,
  UnknownAttachment,
  UnknownMessage,
  CannotEditMessage,
  CannotJoinCall,
  NotPinned,

  // Server errors
  UnknownServer,
  Banned,
  Spam,

  // Bot errors
  IsBot,
  IsNotBot,
  BotIsPrivate,

  // Permission errors
  MissingPermission { permission: String },
  MissingUserPermission { permission: String },
  NotElevated,
  NotPrivileged,
  CannotGiveMissingPermissions,
  NotOwner,
  IsElevated,

  // Database errors
  DatabaseError,
  DBConnectionError,
  DBSelectError,
  DBInsertError,
  DBUpdateError,
  DBDeleteError,
  ForeignKeyViolation,

  // External service errors
  InternalError,
  Connection,
  Privileges,
  ConfigError,
  HttpRequestError,
  HttpResponseError,
  HttpEmptyResponse,
  ProxyError,
  LiveKitUnavailable,
  VosoUnavailable,

  // Voice/WebRTC errors
  NotAVoiceChannel,
  NotConnected,
  UnknownNode,

  // File & media errors
  FileTypeNotAllowed,
  ImageProcessingFailed,
  MissingField,
  InvalidData,

  // Task & async errors
  TimedOut,
  TaskFailed,

  // Feature flags
  FeatureDisabled { feature: String },

  // JSON errors
  JsonMarshal,
  JsonUnmarshal,
}

impl fmt::Display for ErrorType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ErrorType::LabelMe => write!(f, "This error was not labeled"),
      ErrorType::AlreadyOnboarded => write!(f, "User is already onboarded"),
      ErrorType::InvalidOperation => write!(f, "Invalid operation"),
      ErrorType::InvalidCredentials => write!(f, "Invalid credentials"),
      ErrorType::InvalidProperty => write!(f, "Invalid property"),
      ErrorType::InvalidSession => write!(f, "Invalid session"),
      ErrorType::InvalidFlagValue => write!(f, "Invalid flag value"),
      ErrorType::NotAuthenticated => write!(f, "Not authenticated"),
      ErrorType::DuplicateNonce => write!(f, "Duplicate nonce"),
      ErrorType::NotFound => write!(f, "Resource not found"),
      ErrorType::NoEffect => write!(f, "Operation had no effect"),
      ErrorType::NoRows => write!(f, "No rows returned"),
      ErrorType::NotNullViolation => write!(f, "NOT NULL constraint violation"),
      ErrorType::NoEmbedData => write!(f, "No embed data available"),
      ErrorType::EmptyMessage => write!(f, "Message cannot be empty"),
      ErrorType::PayloadTooLarge => write!(f, "Payload too large"),
      ErrorType::FailedValidation { error } => write!(f, "Validation failed: {}", error),
      ErrorType::InvalidUsername => write!(f, "Invalid username"),
      ErrorType::InvalidRole => write!(f, "Invalid role"),
      ErrorType::InvalidNumber => write!(f, "Invalid number"),
      ErrorType::Base64Invalid => write!(f, "Invalid Base64 data"),
      ErrorType::RegexInvalid => write!(f, "Invalid regex pattern"),
      ErrorType::DiscriminatorChangeRatelimited => write!(f, "Discriminator change rate limited"),
      ErrorType::TooManyAttachments { max } => write!(f, "Too many attachments (max: {})", max),
      ErrorType::TooManyEmbeds { max } => write!(f, "Too many embeds (max: {})", max),
      ErrorType::TooManyReplies { max } => write!(f, "Too many replies (max: {})", max),
      ErrorType::TooManyChannels { max } => write!(f, "Too many channels (max: {})", max),
      ErrorType::TooManyServers { max } => write!(f, "Too many servers (max: {})", max),
      ErrorType::TooManyEmoji { max } => write!(f, "Too many emoji (max: {})", max),
      ErrorType::TooManyRoles { max } => write!(f, "Too many roles (max: {})", max),
      ErrorType::GroupTooLarge { max } => write!(f, "Group too large (max: {})", max),
      ErrorType::ReachedMaximumBots => write!(f, "Reached maximum number of bots"),
      ErrorType::FileTooLarge { max } => write!(f, "File too large (max: {} bytes)", max),
      ErrorType::FileTooSmall => write!(f, "File too small"),
      ErrorType::UsernameTaken => write!(f, "Username is already taken"),
      ErrorType::AlreadyFriends => write!(f, "Users are already friends"),
      ErrorType::AlreadySentRequest => write!(f, "Friend request already sent"),
      ErrorType::AlreadyInGroup => write!(f, "Already in group"),
      ErrorType::AlreadyInServer => write!(f, "Already in server"),
      ErrorType::AlreadyPinned => write!(f, "Message is already pinned"),
      ErrorType::AlreadyConnected => write!(f, "Already connected"),
      ErrorType::UniqueViolation => write!(f, "Unique constraint violation"),
      ErrorType::ResourceExists => write!(f, "Resource already exists"),
      ErrorType::UnknownUser => write!(f, "Unknown user"),
      ErrorType::Blocked => write!(f, "You have blocked this user"),
      ErrorType::BlockedByOther => write!(f, "You are blocked by this user"),
      ErrorType::NotFriends => write!(f, "Users are not friends"),
      ErrorType::NotInGroup => write!(f, "Not in group"),
      ErrorType::CannotRemoveYourself => write!(f, "Cannot remove yourself"),
      ErrorType::CannotTimeoutYourself => write!(f, "Cannot timeout yourself"),
      ErrorType::CannotReportYourself => write!(f, "Cannot report yourself"),
      ErrorType::TooManyPendingFriendRequests { max } => {
        write!(f, "Too many pending friend requests (max: {})", max)
      }
      ErrorType::UnknownChannel => write!(f, "Unknown channel"),
      ErrorType::UnknownAttachment => write!(f, "Unknown attachment"),
      ErrorType::UnknownMessage => write!(f, "Unknown message"),
      ErrorType::CannotEditMessage => write!(f, "Cannot edit message"),
      ErrorType::CannotJoinCall => write!(f, "Cannot join call"),
      ErrorType::NotPinned => write!(f, "Message is not pinned"),
      ErrorType::UnknownServer => write!(f, "Unknown server"),
      ErrorType::Banned => write!(f, "Banned"),
      ErrorType::Spam => write!(f, "Marked as spam"),
      ErrorType::IsBot => write!(f, "User is a bot"),
      ErrorType::IsNotBot => write!(f, "User is not a bot"),
      ErrorType::BotIsPrivate => write!(f, "Bot is private"),
      ErrorType::MissingPermission { permission } => {
        write!(f, "Missing permission: {}", permission)
      }
      ErrorType::MissingUserPermission { permission } => {
        write!(f, "Missing user permission: {}", permission)
      }
      ErrorType::NotElevated => write!(f, "Not elevated"),
      ErrorType::NotPrivileged => write!(f, "Not privileged"),
      ErrorType::CannotGiveMissingPermissions => write!(f, "Cannot give missing permissions"),
      ErrorType::NotOwner => write!(f, "Not owner"),
      ErrorType::IsElevated => write!(f, "Is elevated"),
      ErrorType::DatabaseError => write!(f, "Database error"),
      ErrorType::DBConnectionError => write!(f, "Database connection error"),
      ErrorType::DBSelectError => write!(f, "Database select error"),
      ErrorType::DBInsertError => write!(f, "Database insert error"),
      ErrorType::DBUpdateError => write!(f, "Database update error"),
      ErrorType::DBDeleteError => write!(f, "Database delete error"),
      ErrorType::ForeignKeyViolation => write!(f, "Foreign key constraint violation"),
      ErrorType::InternalError => write!(f, "Internal error"),
      ErrorType::Connection => write!(f, "Connection error"),
      ErrorType::Privileges => write!(f, "Insufficient privileges"),
      ErrorType::ConfigError => write!(f, "Configuration error"),
      ErrorType::HttpRequestError => write!(f, "HTTP request error"),
      ErrorType::HttpResponseError => write!(f, "HTTP response error"),
      ErrorType::HttpEmptyResponse => write!(f, "Empty HTTP response"),
      ErrorType::ProxyError => write!(f, "Proxy error"),
      ErrorType::LiveKitUnavailable => write!(f, "LiveKit service unavailable"),
      ErrorType::VosoUnavailable => write!(f, "Voso service unavailable"),
      ErrorType::NotAVoiceChannel => write!(f, "Not a voice channel"),
      ErrorType::NotConnected => write!(f, "Not connected"),
      ErrorType::UnknownNode => write!(f, "Unknown node"),
      ErrorType::FileTypeNotAllowed => write!(f, "File type not allowed"),
      ErrorType::ImageProcessingFailed => write!(f, "Image processing failed"),
      ErrorType::MissingField => write!(f, "Missing required field"),
      ErrorType::InvalidData => write!(f, "Invalid data"),
      ErrorType::TimedOut => write!(f, "Operation timed out"),
      ErrorType::TaskFailed => write!(f, "Task failed"),
      ErrorType::FeatureDisabled { feature } => write!(f, "Feature disabled: {}", feature),
      ErrorType::JsonMarshal => write!(f, "JSON marshaling error"),
      ErrorType::JsonUnmarshal => write!(f, "JSON unmarshaling error"),
    }
  }
}
