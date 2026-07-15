CREATE TABLE IF NOT EXISTS servers (
  id text PRIMARY KEY, -- ULID
  
  -- Basic info
  owner_id text,
  name text,
  description text,
  
  -- Default set of server and channel permissions
  default_permissions bigint,
  
  icon frozen<file_type>,
  banner frozen<file_type>,
  
  -- Settings
  flags int,
  nsfw boolean,
  
  -- Whether to enable analytics
  analytics boolean,
  -- Whether this server should be publicly discoverable
  discoverable boolean,
  
  -- Roles for this server
  roles map<text,frozen<role_type>>,
  categories list<frozen<category_type>>,
  
  -- Configuration for sending system event messages
  system_messages frozen<system_messages_type>,
  
  stats frozen<server_stats_type>,
  
  -- Channels within this server
  channels list<text>,
  
  created_at bigint,
  updated_at bigint
);
