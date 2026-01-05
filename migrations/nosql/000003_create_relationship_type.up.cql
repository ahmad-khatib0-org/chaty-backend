CREATE TYPE IF NOT EXISTS relationship_type (
  target_id text, -- ULID
  status text, -- 'none, 'friend', 'user', 'outgoing', 'incoming', 'blocked', 'blocked_other'.
  created_at timestamp
);
