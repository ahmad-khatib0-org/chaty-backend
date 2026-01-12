-- System message: Message Pinned
CREATE TYPE IF NOT EXISTS message_system_message_pinned_type (
  id text, -- Message id
  "by" text -- Who pinned it
);
