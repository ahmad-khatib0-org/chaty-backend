use chaty_proto::File;
use scylla::{value::CqlTimestamp, DeserializeValue, SerializeValue};

#[derive(SerializeValue, DeserializeValue, Debug, Clone)]
pub struct FileDB {
  pub id: String,
  pub uploader_id: String,
  pub bucket: String,
  pub filename: String,
  pub content_type: String,
  pub size: i64,
  pub hash: String,
  pub uploaded_at: CqlTimestamp,
  pub deleted: Option<bool>,
  pub reported: Option<bool>,
}

impl From<FileDB> for File {
  fn from(f: FileDB) -> Self {
    Self {
      id: f.id,
      uploader_id: f.uploader_id,
      bucket: f.bucket,
      filename: f.filename,
      content_type: f.content_type,
      size: f.size,
      hash: f.hash,
      uploaded_at: f.uploaded_at.0,
      deleted: f.deleted,
      reported: f.reported,
    }
  }
}
