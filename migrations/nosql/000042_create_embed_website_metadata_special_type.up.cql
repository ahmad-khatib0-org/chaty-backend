-- Website metadata special (oneof)
CREATE TYPE IF NOT EXISTS embed_website_metadata_special_type (
  "none" boolean, -- true if no remote content
  gif boolean, -- true if contains GIF
  youtube frozen<embed_youtube_type>,
  lightspeed frozen<embed_lightspeed_type>,
  twitch frozen<embed_twitch_type>,
  spotify frozen<embed_spotify_type>,
  soundcloud boolean, -- true if Soundcloud
  bandcamp frozen<embed_bandcamp_type>,
  apple_music frozen<embed_apple_music_type>,
  streamable frozen<embed_streamable_type>
);
