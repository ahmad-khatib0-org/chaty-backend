CREATE TABLE IF NOT EXISTS users (
  id VARCHAR(26) PRIMARY KEY,
  username VARCHAR(64) NOT NULL,
  email VARCHAR(255) NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  display_name VARCHAR(64),
  badges INT DEFAULT 0,
  status_text VARCHAR(510),
  status_presence VARCHAR(32) DEFAULT 'USER_STATUS_ONLINE',
  profile_content TEXT,
  profile_background_id VARCHAR(26),
  privileged BOOLEAN DEFAULT FALSE,
  suspended_until BIGINT,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL,
  verified BOOLEAN DEFAULT FALSE
);

CREATE UNIQUE INDEX users_username_idx ON users (username);

CREATE UNIQUE INDEX users_email_idx ON users (email);

CREATE INDEX users_created_at_idx ON users (created_at);

CREATE INDEX users_verified_idx ON users (verified);
