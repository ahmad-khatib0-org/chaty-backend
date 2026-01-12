-- Text Embed
CREATE TYPE IF NOT EXISTS embed_text_type (
  icon_url text, -- URL to icon (optional)
  url text, -- URL for title (optional)
  title text, -- Title of text embed (optional)
  description text, -- Description of text embed (optional)
  media frozen<file_type>, -- ID of uploaded autumn file (optional)
  colour text -- CSS Colour (optional)
);
