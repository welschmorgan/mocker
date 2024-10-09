use std::ops::{Deref, DerefMut};

use crate::{Buffer, Error, ErrorKind, StartLine, Status, Version};

#[derive(Clone, Default)]
pub struct Response(Buffer);

#[cfg(feature = "json")]
impl Response {
  pub fn json<B: serde::Serialize>(status: Status, body: &B) -> crate::Result<Self> {
    let json = serde_json::to_string_pretty(body)?;
    Ok(
      Self::default()
        .with_status_code(status.code())
        .with_header("Content-Type", "application/json")
        .with_body(json),
    )
  }
}

#[cfg(feature = "toml")]
impl Response {
  pub fn toml<B: serde::Serialize>(status: Status, body: &B) -> crate::Result<Self> {
    let toml = toml::to_string_pretty(body)?;
    Ok(
      Self::default()
        .with_status_code(status.code())
        .with_header("Content-Type", "application/toml")
        .with_body(toml),
    )
  }
}

#[cfg(feature = "yaml")]
impl Response {
  pub fn yaml<B: serde::Serialize>(status: Status, body: &B) -> crate::Result<Self> {
    let yaml = serde_yml::to_string(body)?;
    Ok(
      Self::default()
        .with_status_code(status.code())
        .with_header("Content-Type", "application/yaml")
        .with_body(yaml),
    )
  }
}

impl Response {
  pub fn api<B: serde::Serialize>(status: Status, body: &B) -> crate::Result<Self> {
    #[cfg(feature = "json")]
    return Self::json(status, body);
    #[cfg(feature = "toml")]
    return Self::toml(status, body);
    #[cfg(feature = "yaml")]
    return Self::yaml(status, body);
    Err(Error::new(
      ErrorKind::Api(Status::InternalServerError),
      Some(format!(
        "no api format defined: please select either `json`, `toml` or `yaml` feature"
      )),
      None,
    ))
  }

  pub fn with_status(mut self, status: Status) -> Self {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.status = status.code();
    res.reason = Some(status.text().to_string());
    self
  }

  pub fn with_status_code(mut self, code: u16) -> Self {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.status = code;
    res.reason = Status::try_from(code)
      .ok()
      .map(|status| status.descr().1.to_string());
    self
  }

  pub fn with_version(mut self, version: Version) -> Self {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.version = version;
    self
  }

  pub fn with_reason<R: AsRef<str>>(mut self, r: R) -> Self {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.reason = Some(r.as_ref().to_string());
    self
  }

  pub fn with_headers<K: AsRef<str>, V: AsRef<str>, I: IntoIterator<Item = (K, V)>>(
    mut self,
    v: I,
  ) -> Self {
    self.0 = self.0.with_headers(v);
    self
  }
  pub fn with_header<K: AsRef<str>, V: AsRef<str>>(mut self, k: K, v: V) -> Self {
    self.0 = self.0.with_header(k, v);
    self
  }
  pub fn with_body<B: AsRef<str>>(mut self, v: B) -> Self {
    self.0 = self.0.with_body(v);
    self
  }
  pub fn append_body<B: AsRef<str>>(&mut self, v: B) {
    self.0.append_body(v);
  }
  pub fn set_header<K: AsRef<str>, V: AsRef<str>>(&mut self, k: K, v: V) {
    self.0.set_header(k, v);
  }
}

unsafe impl Send for Response {}
unsafe impl Sync for Response {}

impl Deref for Response {
  type Target = Buffer;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Response {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<Error> for Response {
  fn from(value: Error) -> Self {
    let status = match value.kind() {
      ErrorKind::Api(status) => status,
      _ => Status::InternalServerError,
    };
    let mut res = Response::default().with_status_code(status.code());
    if let Some(msg) = value.message() {
      res = res.with_body(msg);
    }
    res
  }
}
