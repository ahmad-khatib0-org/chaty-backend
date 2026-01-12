-- System message: Message Unpinned
CREATE TYPE IF NOT EXISTS message_system_message_unpinned_type (
  id text, -- Message id
  "by" text -- Who unpinned it
);
