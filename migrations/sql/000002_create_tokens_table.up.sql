CREATE TABLE IF NOT EXISTS tokens (
  id VARCHAR(26) PRIMARY KEY,
  user_id VARCHAR NOT NULL,
  token VARCHAR(256) NOT NULL,
  type VARCHAR(64) NOT NULL, -- E,g email_verification, password_reset ...
  used BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL,
  expires_at BIGINT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS tokens_user_id_type_idx ON tokens (user_id, type);
