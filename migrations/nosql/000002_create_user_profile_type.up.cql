-- User profile
CREATE TYPE IF NOT EXISTS user_profile_type (
  content text,
  background_id text -- ULID
);
