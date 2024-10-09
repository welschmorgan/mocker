use std::{
  io::Read,
  ops::{Deref, DerefMut},
};

use crate::{Buffer, Method};

#[derive(Clone, Default)]
pub struct Request(Buffer);

impl Request {
  const BUF_SIZE: usize = 255;

  pub fn from_reader<R: Read>(mut r: R) -> crate::Result<Self> {
    let mut block: [u8; Self::BUF_SIZE] = [0u8; Self::BUF_SIZE];
    let mut buf = vec![];
    loop {
      let nread = r.read(&mut block)?;
      buf.extend_from_slice(&block[0..nread]);
      if nread < Self::BUF_SIZE {
        break;
      }
    }
    let s = std::str::from_utf8(&buf)?;
    Ok(Self(s.parse::<Buffer>()?))
  }

  pub fn query_param<K: AsRef<str>>(&self, k: K) -> Option<(String, Option<String>)> {
    match self
      .query_params()
      .iter()
      .find(|(key, _val)| key.eq_ignore_ascii_case(k.as_ref()))
    {
      Some((key, val)) => Some((key.clone(), val.clone())),
      None => None,
    }
  }

  pub fn query_params(&self) -> Vec<(String, Option<String>)> {
    let query = match self.query() {
      Some(q) => q,
      None => return vec![],
    };
    query
      .split("&")
      .map(|param| match param.split_once('=') {
        Some((key, val)) => (key.to_string(), Some(val.to_string())),
        None => (param.to_string(), None),
      })
      .collect::<Vec<_>>()
  }

  pub fn query(&self) -> Option<&str> {
    let start = self.start_line().as_request().unwrap();
    match start.target.split_once('?') {
      Some((first, second)) => Some(second),
      None => None,
    }
  }

  pub fn method(&self) -> Option<Method> {
    self.start_line().as_request().map(|r| r.method)
  }

  pub fn path(&self) -> Option<&str> {
    let start = self.start_line().as_request().unwrap();
    match start.target.split_once('?') {
      Some((first, second)) => Some(first),
      None => None,
    }
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

unsafe impl Send for Request {}
unsafe impl Sync for Request {}

impl Deref for Request {
  type Target = Buffer;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Request {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
