-- Spotify track
CREATE TYPE IF NOT EXISTS embed_spotify_type (
  content_type text, -- Free-form string
  id text
);
