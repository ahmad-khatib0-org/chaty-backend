CREATE TYPE IF NOT EXISTS embed_image_type (
  url text, -- URL to the original image
  width int, -- Width of the image
  height int, -- Height of the image
  size text -- Positioning and size: "large" or "preview"
);
