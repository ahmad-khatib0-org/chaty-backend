#[derive(Clone, Debug, Default)]
pub struct Session {
  pub id: String,
  pub token: String,
  pub created_at: i64,
  pub expires_at: i64,
  pub last_activity_at: i64,
  pub user_id: String,
  pub device_id: String,
}

impl Session {
  pub fn id(&self) -> &str {
    &self.id
  }
  pub fn token(&self) -> &str {
    &self.token
  }
  pub fn created_at(&self) -> f64 {
    self.created_at as f64
  }
  pub fn expires_at(&self) -> f64 {
    self.expires_at as f64
  }
  pub fn last_activity_at(&self) -> f64 {
    self.last_activity_at as f64
  }
  pub fn user_id(&self) -> &str {
    &self.user_id
  }
  pub fn device_id(&self) -> &str {
    &self.device_id
  }
}

#[derive(Clone, Debug, Default)]
pub struct Context {
  pub session: Session,
  pub request_id: String,
  pub ip_address: String,
  pub x_forwarded_for: String,
  pub path: String,
  pub user_agent: String,
  pub accept_language: String,
  pub timezone: String,
}

impl Context {
  pub fn new(
    session: Session,
    request_id: String,
    ip_address: String,
    x_forwarded_for: String,
    path: String,
    user_agent: String,
    accept_language: String,
    timezone: String,
  ) -> Self {
    Self {
      session,
      request_id,
      ip_address,
      x_forwarded_for,
      path,
      user_agent,
      accept_language,
      timezone,
    }
  }

  pub fn clone(&self) -> Self {
    Self {
      session: self.session.clone(),
      request_id: self.request_id.clone(),
      ip_address: self.ip_address.clone(),
      x_forwarded_for: self.x_forwarded_for.clone(),
      path: self.path.clone(),
      user_agent: self.user_agent.clone(),
      accept_language: self.accept_language.clone(),
      timezone: self.timezone.clone(),
    }
  }

  pub fn session(&self) -> Session {
    self.session.clone()
  }
  pub fn request_id(&self) -> &str {
    &self.request_id
  }
  pub fn ip_address(&self) -> &str {
    &self.ip_address
  }
  pub fn x_forwarded_for(&self) -> &str {
    &self.x_forwarded_for
  }
  pub fn path(&self) -> &str {
    &self.path
  }
  pub fn user_agent(&self) -> &str {
    &self.user_agent
  }
  pub fn accept_language(&self) -> &str {
    &self.accept_language
  }
}
