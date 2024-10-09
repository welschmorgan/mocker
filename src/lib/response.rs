use std::ops::{Deref, DerefMut};

use crate::{Buffer, StartLine, Status, Version};

#[derive(Clone, Default)]
pub struct Response(Buffer);

impl Response {
  pub fn with_status(mut self, code: u16) -> Response {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.status = code;
    res.reason = Status::try_from(code)
      .ok()
      .map(|status| status.descr().1.to_string());
    self
  }

  pub fn with_version(mut self, version: Version) -> Response {
    let res = self.0.start_line_mut().as_response_mut().unwrap();
    res.version = version;
    self
  }

  pub fn with_reason<R: AsRef<str>>(mut self, r: R) -> Response {
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
