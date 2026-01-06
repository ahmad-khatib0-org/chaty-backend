CREATE TYPE IF NOT EXISTS channel_group_type (
  user_id text, -- User id of the owner of the group
  name text, -- Display name of the channel
  description text, -- Channel description
  recipients frozen<set<text>>, -- Array of user ids participating in channel
  icon frozen<file_type>, -- Custom icon attachment
  last_message_id text, -- Id of the last message sent in this channel
  -- Permissions assigned to members of this group
  -- (does not apply to the owner of the group)
  permissions bigint,
  nsfw boolean -- Whether this group is marked as not safe for work
);
