-- Twitch stream or clip
CREATE TYPE IF NOT EXISTS embed_twitch_type (
  content_type text, -- "channel", "video", "clip"
  id text
);
