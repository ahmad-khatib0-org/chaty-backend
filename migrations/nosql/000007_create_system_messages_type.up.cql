-- System messages config
CREATE TYPE IF NOT EXISTS system_messages_type (
  user_joined_channel_id text,
  user_left_channel_id text,
  user_kicked_channel_id text,
  user_banned_channel_id text
);
