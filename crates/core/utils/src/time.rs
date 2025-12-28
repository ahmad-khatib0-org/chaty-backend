use chrono::{NaiveDate, Utc};

pub fn time_get_millis() -> u64 {
  Utc::now().timestamp_millis().try_into().unwrap()
}

pub fn time_get_seconds() -> u64 {
  Utc::now().timestamp().try_into().unwrap()
}

pub fn date_to_milliseconds(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
  let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
  let naive_datetime = naive_date.and_hms_opt(0, 0, 0).ok_or("Invalid time")?;
  Ok(naive_datetime.and_utc().timestamp_millis().to_string())
}
