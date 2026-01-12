-- System message: User Added
CREATE TYPE IF NOT EXISTS message_system_user_added_type (
  id text, -- User id
  "by" text -- Who added the user
);
