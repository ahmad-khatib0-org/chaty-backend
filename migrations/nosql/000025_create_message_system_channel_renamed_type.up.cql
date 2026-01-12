-- System message: Channel Renamed
CREATE TYPE IF NOT EXISTS message_system_channel_renamed_type (
  name text, -- New channel name
  "by" text -- Who renamed it
);
