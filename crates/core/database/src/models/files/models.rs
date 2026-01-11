use scylla::{value::CqlTimestamp, DeserializeValue, SerializeValue};

#[derive(SerializeValue, DeserializeValue, Debug)]
pub struct FileDB {
  pub id: String,
  pub uploader_id: String,
  pub bucket: String,
  pub filename: String,
  pub content_type: String,
  pub size: i64,
  pub hash: String,
  pub uploaded_at: CqlTimestamp, // Matches CQL Native(Timestamp)
  pub deleted: Option<bool>,
  pub reported: Option<bool>,
}
