-- System message: User Remove
CREATE TYPE IF NOT EXISTS message_system_user_remove_type (
  id text, -- User id
  "by" text -- Who removed the user
);
