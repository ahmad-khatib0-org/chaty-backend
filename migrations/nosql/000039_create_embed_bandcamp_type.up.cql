-- Bandcamp track
CREATE TYPE IF NOT EXISTS embed_bandcamp_type (
  content_type text, -- "album", "track"
  id text
);
