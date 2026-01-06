CREATE TYPE IF NOT EXISTS channel_direct_message_type (
  -- Whether this direct message channel is currently open on both sides 
  active boolean, 
  -- 2-tuple of user ids participating in direct message
  recipients set<text>,
  -- Id of the last message sent in this channel
  last_message_id text
);
