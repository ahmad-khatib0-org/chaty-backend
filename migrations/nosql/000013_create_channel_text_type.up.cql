CREATE TYPE IF NOT EXISTS channel_text_type (
  -- Id of the server this channel belongs to
  server_id text,
  -- Display name of the channel
  name text,
  -- Channel description (optional)
  description text,
  -- Custom icon attachment (optional)
  icon frozen<file_type>,
  -- Id of the last message sent in this channel (optional)
  last_message_id text,
  -- Default permissions assigned to users in this channel (optional)
  default_permissions frozen<override_field_type>,
  -- Permissions assigned based on role to this channel
  -- Map of role_id -> permissions
  role_permissions map<text, frozen<override_field_type>>,
  -- Whether this channel is marked as not safe for work
  nsfw boolean
);
