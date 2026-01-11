CREATE TABLE IF NOT EXISTS channels_by_user(
  user_id text, -- ULID
  channel_id text, -- ULID
  channel_type text, -- 'saved_messages', 'direct_message', 'group', 'text_channel'
  -- channel types
  saved frozen<channel_saved_messages_type>,
  direct frozen<channel_direct_message_type>,
  group frozen<channel_group_type>,
  text frozen<channel_text_type>,
  -- Voice settings
  voice_max_users int,
  -- timestamps
  created_at timestamp,
  updated_at timestamp,
  PRIMARY KEY ((user_id), channel_id)
) WITH CLUSTERING ORDER BY (channel_id DESC);
