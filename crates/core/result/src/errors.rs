use std::fmt;

#[derive(Debug, Clone)]
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
  DatabaseError { operation: String, collection: String },
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
      ErrorType::DatabaseError { operation, collection } => {
        write!(f, "Database error during {} on {}", operation, collection)
      }
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
