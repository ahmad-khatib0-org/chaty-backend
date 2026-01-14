use chaty_permission::ChannelType;

use crate::EnumHelpers;

impl EnumHelpers for ChannelType {
  fn to_str(&self) -> &'static str {
    match self {
      ChannelType::SavedMessages => "saved_messages",
      ChannelType::DirectMessage => "direct_message",
      ChannelType::Group => "group",
      ChannelType::ServerChannel => "text_channel",
      ChannelType::Unknown => unimplemented!(),
    }
  }

  fn from_optional_string(value: Option<String>) -> Option<Self> {
    match value.as_deref() {
      Some("saved_messages") => Some(ChannelType::SavedMessages),
      Some("direct_message") => Some(ChannelType::DirectMessage),
      Some("group") => Some(ChannelType::Group),
      Some("text_channel") => Some(ChannelType::ServerChannel),
      _ => None,
    }
  }
}
