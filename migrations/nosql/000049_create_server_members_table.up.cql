CREATE TABLE IF NOT EXISTS server_members (
  -- Partition: All members of a server
  server_id text,
  user_id text,
  
  -- User info (denormalized)
  username text,
  avatar frozen <file_type>,
  
  -- Member-specific
  nickname text,
  joined_at bigint, -- int64 timestamp
  
  roles set<text>, -- Role IDs
  timeout bigint,
  
  can_publish boolean,
  can_receive boolean,
  
  PRIMARY KEY ((server_id), user_id)
);
