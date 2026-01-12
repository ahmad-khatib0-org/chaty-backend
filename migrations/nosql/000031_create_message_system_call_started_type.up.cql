-- System message: Call Started
CREATE TYPE IF NOT EXISTS message_system_call_started_type (
  "by" text, -- Who started call
  finished_at timestamp -- When call finished (optional)
);
