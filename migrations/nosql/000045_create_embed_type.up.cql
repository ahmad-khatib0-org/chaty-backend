-- Embed (oneof - using separate optional fields)
CREATE TYPE IF NOT EXISTS embed_type (
  website frozen<embed_website_metadata_type>,
  image frozen<embed_image_type>,
  video frozen<embed_video_type>,
  text frozen<embed_text_type>,
  "none" boolean -- true if no embed
);
