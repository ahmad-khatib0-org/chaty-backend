CREATE TYPE IF NOT EXISTS message_webhook_type (
  name text, -- The name of the webhook - 1 to 32 chars
  avatar text -- The id of the avatar of the webhook, if it has one (optional)
);
