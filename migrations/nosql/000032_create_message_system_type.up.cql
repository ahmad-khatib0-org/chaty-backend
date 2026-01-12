-- ===========================================
-- MESSAGE SYSTEM (Main type using frozen variants)
-- ==========================================
CREATE TYPE IF NOT EXISTS message_system_type (
  text frozen<message_system_text_type>,
  user_added frozen<message_system_user_added_type>,
  user_remove frozen<message_system_user_remove_type>,
  user_joined frozen<message_system_user_joined_type>,
  user_left frozen<message_system_user_left_type>,
  user_kicked frozen<message_system_user_kicked_type>,
  user_banned frozen<message_system_user_banned_type>,
  channel_renamed frozen<message_system_channel_renamed_type>,
  channel_description_changed frozen<message_system_channel_description_changed_type>,
  channel_icon_changed frozen<message_system_channel_icon_changed_type>,
  channel_ownership_changed frozen<message_system_channel_ownership_changed_type>,
  message_pinned frozen<message_system_message_pinned_type>,
  message_unpinned frozen<message_system_message_unpinned_type>,
  call_started frozen<message_system_call_started_type>
);
