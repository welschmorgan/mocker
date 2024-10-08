use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
  IO,
  Unknown,
}

#[derive(Debug)]
pub struct Error {
  kind: ErrorKind,
  message: Option<String>,
  cause: Option<Box<dyn std::error::Error>>,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl Error {
  pub fn new(
    kind: ErrorKind,
    msg: Option<String>,
    cause: Option<Box<dyn std::error::Error>>,
  ) -> Self {
    Self {
      kind,
      message: msg,
      cause,
    }
  }

  pub fn kind_as_str(&self) -> &'static str {
    match self.kind {
      ErrorKind::IO => "i/o",
      ErrorKind::Unknown => "unknown",
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\x1b[1m{}\x1b[0m{}{}",
      self.kind_as_str(),
      match self.message.as_ref() {
        Some(msg) => format!(": {}", msg),
        None => String::new(),
      },
      match self.cause.as_ref() {
        Some(cause) => format!(". Caused by: {}", cause),
        None => String::new(),
      }
    )
  }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Error::new(ErrorKind::IO, Some(value.to_string()), None)
  }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
  fn from(value: serde_json::Error) -> Self {
    Error::new(ErrorKind::IO, Some(value.to_string()), None)
  }
}

#[cfg(feature = "toml")]
impl From<toml::ser::Error> for Error {
  fn from(value: toml::ser::Error) -> Self {
    Error::new(ErrorKind::IO, Some(value.to_string()), None)
  }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for Error {
  fn from(value: toml::de::Error) -> Self {
    Error::new(ErrorKind::IO, Some(value.to_string()), None)
  }
}

#[cfg(feature = "yaml")]
impl From<serde_yml::Error> for Error {
  fn from(value: serde_yml::Error) -> Self {
    Error::new(ErrorKind::IO, Some(value.to_string()), None)
  }
}

impl From<Box<dyn std::error::Error>> for Error {
  fn from(value: Box<dyn std::error::Error>) -> Self {
    Error::new(ErrorKind::Unknown, Some(value.to_string()), None)
  }
}