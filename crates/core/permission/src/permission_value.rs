use crate::{ChannelPermission, Override, UserPermission};

/// Holds a permission value to manipulate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PermissionValue(u64);

impl PermissionValue {
  pub fn from_raw(value: u64) -> Self {
    Self(value)
  }

  pub fn into_raw(self) -> u64 {
    self.0
  }

  /// Apply a given override to this value
  pub fn apply(&mut self, v: Override) {
    self.allow(v.allow);
    self.revoke(v.deny);
  }

  /// Allow given permissions
  pub fn allow(&mut self, v: u64) {
    self.0 |= v;
  }

  /// Revoke given permissions
  pub fn revoke(&mut self, v: u64) {
    self.0 &= !v;
  }

  /// Revoke all permissions
  pub fn revoke_all(&mut self) {
    self.0 = 0;
  }

  /// Restrict to given permissions
  pub fn restrict(&mut self, v: u64) {
    self.0 &= v;
  }

  /// Check whether certain a permission has been granted
  pub fn has(&self, v: u64) -> bool {
    (self.0 & v) == v
  }

  /// Check whether certain a user permission has been granted
  pub fn has_user_permission(&self, permission: UserPermission) -> bool {
    self.has(permission as u64)
  }

  /// Check whether certain a channel permission has been granted
  pub fn has_channel_permission(&self, permission: ChannelPermission) -> bool {
    self.has(permission as u64)
  }
}

impl From<i64> for PermissionValue {
  fn from(v: i64) -> Self {
    Self(v as u64)
  }
}

impl From<u64> for PermissionValue {
  fn from(v: u64) -> Self {
    Self(v)
  }
}

impl From<PermissionValue> for u64 {
  fn from(v: PermissionValue) -> Self {
    v.0
  }
}

impl From<ChannelPermission> for PermissionValue {
  fn from(v: ChannelPermission) -> Self {
    (v as u64).into()
  }
}
