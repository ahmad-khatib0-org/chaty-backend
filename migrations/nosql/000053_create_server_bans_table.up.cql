CREATE TABLE IF NOT EXISTS server_bans (
  server_id text, --- ULID
  user_id text,
  reason text,
  banned_by text, --- user id
  banned_at bigint,
  PRIMARY KEY ((server_id), user_id)
);
