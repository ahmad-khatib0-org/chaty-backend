-- Role (for server roles)
CREATE TYPE IF NOT EXISTS role_type (
  name text,
  permissions_allow bigint,
  permissions_deny bigint,
  colour text, -- '#FF0000' or null
  hoist boolean,
  rank int,
  created_at timestamp,
  updated_at timestamp
);
