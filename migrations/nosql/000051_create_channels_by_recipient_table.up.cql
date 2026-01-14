CREATE TABLE IF NOT EXISTS channels_by_recipient (
  -- Recipient's user ID
  recipient_user_id text,
  -- Channel ID
  channel_id text,
  -- Channel type: -- 'saved_messages', 'direct_message', 'group', 'text_channel'
  channel_type text,
  -- When the channel was created
  created_at timestamp,
  -- Primary key for fast lookups by recipient
  PRIMARY KEY ((recipient_user_id), channel_id)
) WITH CLUSTERING ORDER BY (channel_id DESC);
