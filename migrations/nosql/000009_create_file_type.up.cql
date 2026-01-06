CREATE TYPE IF NOT EXISTS file_type (
  id text,
  uploader_id text, -- ID of user who uploaded this file
  bucket text, -- Tag / bucket this file was uploaded to
  filename text, -- Original filename
  content_type text, -- Raw content type of this file
  size bigint, -- Size of this file (in bytes)
  hash text, -- Hash of this file
  uploaded_at timestamp, -- When this file was uploaded
  deleted boolean, -- Whether this file was deleted
  reported boolean -- Whether this file was reported
)
