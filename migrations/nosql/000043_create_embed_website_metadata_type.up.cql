-- Website metadata
CREATE TYPE IF NOT EXISTS embed_website_metadata_type (
  url text, -- Direct URL to web page (optional)
  original_url text, -- Original direct URL (optional)
  special frozen<embed_website_metadata_special_type>, -- Remote content (optional)
  title text, -- Title of website (optional)
  description text, -- Description of website (optional)
  image frozen<embed_image_type>, -- Embedded image (optional)
  video frozen<embed_video_type>, -- Embedded video (optional)
  site_name text, -- Site name (optional)
  icon_url text, -- URL to site icon (optional)
  colour text -- CSS Colour (optional)
);
