use std::{fmt, ops::Add};

/// User's relationship with another user (or themselves)
pub enum RelationshipStatus {
  None,
  User,
  Friend,
  Outgoing,
  Incoming,
  Blocked,
  BlockedOther,
}

/// User permission definitions
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum UserPermission {
  Access = 1 << 0,
  ViewProfile = 1 << 1,
  SendMessage = 1 << 2,
  Invite = 1 << 3,
}

impl fmt::Display for UserPermission {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(self, f)
  }
}

/// UserPermission + UserPermission -> u32
impl Add for UserPermission {
  type Output = u32;

  fn add(self, rhs: UserPermission) -> u32 {
    self as u32 | rhs as u32
  }
}

/// &UserPermission + &UserPermission -> u32
impl Add for &UserPermission {
  type Output = u32;

  fn add(self, rhs: &UserPermission) -> u32 {
    *self as u32 | *rhs as u32
  }
}

/// u32 + UserPermission -> u32
impl Add<UserPermission> for u32 {
  type Output = u32;

  fn add(self, rhs: UserPermission) -> u32 {
    self | rhs as u32
  }
}

/// UserPermission + u32 -> u32
impl Add<u32> for UserPermission {
  type Output = u32;

  fn add(self, rhs: u32) -> u32 {
    self as u32 | rhs
  }
}

/// implement for references
impl Add<&UserPermission> for u32 {
  type Output = u32;

  fn add(self, rhs: &UserPermission) -> u32 {
    self | *rhs as u32
  }
}

impl Add<u32> for &UserPermission {
  type Output = u32;

  fn add(self, rhs: u32) -> u32 {
    *self as u32 | rhs
  }
}
