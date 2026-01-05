CREATE TYPE IF NOT EXISTS user_status_type (
  text text,
  presence text -- 'online', 'idle', 'focus', 'busy', 'invisible'
);
