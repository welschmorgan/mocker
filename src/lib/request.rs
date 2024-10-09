use std::{
  collections::HashMap,
  io::Read,
  ops::{Deref, DerefMut},
};

use serde::{de::DeserializeOwned, Deserialize};

use crate::{Buffer, Error, ErrorKind, Method, Status, Value};

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

  pub fn parse_body<T: DeserializeOwned>(&self) -> crate::Result<T> {
    let body = format!("{}\n", std::str::from_utf8(self.body())?.trim());
    let content_type = match self.header("Content-Type") {
      Some(v) => v,
      None => {
        return Err(Error::new(
          ErrorKind::Api(Status::BadRequest),
          Some(format!("Missing `Content-Type` header")),
          None,
        ));
      }
    };
    #[cfg(feature = "json")]
    if content_type.eq_ignore_ascii_case("application/json") {
      let ret: T = serde_json::from_str(&body).map_err(|e| {
        let mut arrowed_body = body
          .to_string()
          .lines()
          .map(|line| line.to_string())
          .collect::<Vec<_>>();
        let line_id = e.line().min(arrowed_body.len());
        arrowed_body.insert(
          line_id,
          format!(
            "{}\x1b[0;31m⮬\x1b[0m \x1b[1mhere\x1b[0m",
            " ".repeat(e.column() - 1)
          ),
        );
        Error::new(
          ErrorKind::Parse,
          Some(format!(
            "failed to deserialize request body, {}\n--------------------\n{}",
            e,
            arrowed_body.join("\n")
          )),
          None,
        )
      })?;
      return Ok(ret);
    }
    #[cfg(feature = "toml")]
    if content_type.eq_ignore_ascii_case("application/toml") {
      let ret: T = toml::from_str(&body).map_err(|e| {
        Error::new(
          ErrorKind::Parse,
          Some(format!("failed to deserialize request body, {}", e)),
          None,
        )
      })?;
      return Ok(ret);
    }
    #[cfg(feature = "yaml")]
    if content_type.eq_ignore_ascii_case("application/yaml") {
      let ret: T = serde_yml::from_str(&body).map_err(|e| {
        Error::new(
          ErrorKind::Parse,
          Some(format!("failed to deserialize request body, {}", e)),
          None,
        )
      })?;
      return Ok(ret);
    }
    Err(Error::new(
      ErrorKind::Api(Status::InternalServerError),
      Some(format!(
        "Cannot deserialize body of type '{}', missing feature",
        content_type
      )),
      None,
    ))
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
