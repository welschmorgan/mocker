use std::io::Write;

#[derive(Clone)]
pub struct Response {
  status: u16,
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

unsafe impl Send for Response {}
unsafe impl Sync for Response {}

impl Default for Response {
  fn default() -> Self {
    Self {
      status: 200,
      headers: Default::default(),
      body: Default::default(),
    }
  }
}
impl Response {
  pub fn with_status(mut self, v: u16) -> Self {
    self.status = v;
    self
  }

  pub fn with_headers(mut self, v: Vec<(String, String)>) -> Self {
    self.headers = v;
    self
  }

  pub fn with_header<K: AsRef<str>, V: AsRef<str>>(mut self, k: K, v: V) -> Self {
    self
      .headers
      .push((k.as_ref().to_string(), v.as_ref().to_string()));
    self
  }

  pub fn with_body<B: AsRef<str>>(mut self, v: B) -> Self {
    self.body.clear();
    self.append_body(v);
    self
  }

  pub fn append_body<B: AsRef<str>>(&mut self, v: B) {
    let data = v.as_ref().bytes().collect::<Vec<_>>();
    self.body.extend_from_slice(&data);
    self.set_header("Content-Length", self.body.len().to_string());
  }

  pub fn set_header<K: AsRef<str>, V: AsRef<str>>(&mut self, k: K, v: V) {
    match self
      .headers
      .iter_mut()
      .find(|(hk, _hv)| hk.eq_ignore_ascii_case(k.as_ref()))
    {
      Some((_hk, hv)) => *hv = v.as_ref().to_string(),
      None => self
        .headers
        .push((k.as_ref().to_string(), v.as_ref().to_string())),
    }
  }

  pub fn status(&self) -> u16 {
    self.status
  }
  pub fn headers(&self) -> &Vec<(String, String)> {
    &self.headers
  }
  pub fn body(&self) -> &Vec<u8> {
    &self.body
  }

  pub fn write_to<W: Write>(&self, mut w: W) -> crate::Result<()> {
    writeln!(w, "{} {}", self.status(), "foo")?;
    for (key, value) in self.headers() {
      writeln!(w, "{}: {}", key, value)?;
    }
    writeln!(w, "")?;
    w.write(&self.body())?;
    Ok(())
  }
}
