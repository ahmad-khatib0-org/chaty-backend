-- System message: Channel Ownership Changed
CREATE TYPE IF NOT EXISTS message_system_channel_ownership_changed_type (
  "from" text, -- Previous owner
  "to" text -- New owner
);
