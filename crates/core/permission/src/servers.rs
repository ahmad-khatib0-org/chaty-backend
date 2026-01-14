use chaty_proto::OverrideField as OverrideFieldProto;

/// Representation of a single permission override
#[derive(Debug, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Override {
  /// Allow bit flags
  pub allow: u64,
  /// Disallow bit flags
  pub deny: u64,
}

impl Override {
  /// Into allows
  pub fn allows(&self) -> u64 {
    self.allow
  }

  /// Into denies
  pub fn denies(&self) -> u64 {
    self.deny
  }
}

impl From<OverrideField> for Override {
  fn from(v: OverrideField) -> Self {
    Self { allow: v.a as u64, deny: v.d as u64 }
  }
}

impl From<OverrideFieldProto> for Override {
  fn from(v: OverrideFieldProto) -> Self {
    Self { allow: v.allow as u64, deny: v.deny as u64 }
  }
}

impl From<&OverrideFieldProto> for Override {
  fn from(v: &OverrideFieldProto) -> Self {
    Self { allow: v.allow as u64, deny: v.deny as u64 }
  }
}

/// Data permissions Value - contains allow
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPermissionsValue {
  pub permissions: u64,
}

/// Representation of a single permission override
/// as it appears on models and in the database
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OverrideField {
  /// Allow bit flags
  pub a: i64,
  /// Disallow bit flags
  pub d: i64,
}

impl From<Override> for OverrideField {
  fn from(v: Override) -> Self {
    Self { a: v.allow as i64, d: v.deny as i64 }
  }
}

impl From<Override> for OverrideFieldProto {
  fn from(v: Override) -> Self {
    Self { allow: v.allow as i64, deny: v.deny as i64 }
  }
}
