use std::{
  fs::File,
  net::{IpAddr, Ipv4Addr},
  path::{Path, PathBuf},
  str::FromStr,
  sync::{Arc, Mutex},
};

use crate::{config_formats, find_fmt, Error, ErrorKind, Method, Middleware};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub const CONFIG_NAME: &'static str = "mocker.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RouteKind {
  /// A file-backed json store
  #[cfg(feature = "json")]
  Store { path: PathBuf, identifier: String },
  /// A javascript handler
  #[cfg(feature = "js")]
  Script { script: PathBuf, func: String },
}
impl RouteKind {
  pub fn name(&self) -> &'static str {
    match self {
      #[cfg(feature = "json")]
      RouteKind::Store { .. } => "store",
      #[cfg(feature = "js")]
      RouteKind::Script { .. } => "script",
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route(Vec<Method>, String, RouteKind);

impl Route {
  pub fn kind(&self) -> &RouteKind {
    &self.2
  }

  pub fn methods(&self) -> &Vec<Method> {
    &self.0
  }

  pub fn endpoint(&self) -> &String {
    &self.1
  }

  pub fn kind_str(&self) -> &'static str {
    self.kind().name()
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserConfig {
  pub host: Option<IpAddr>,
  pub port: Option<u16>,
  pub middlewares: Option<Vec<String>>,
  pub routes: Vec<Route>,
}

impl UserConfig {
  pub fn realize(&self) -> Config {
    let dflt = Config::default();
    Config {
      host: self.host.unwrap_or_else(|| dflt.host),
      port: self.port.unwrap_or_else(|| dflt.port),
      middlewares: self
        .middlewares
        .as_ref()
        .map(|mws| mws.clone())
        .unwrap_or_default(),
      routes: self.routes.clone(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub host: IpAddr,
  pub port: u16,
  pub middlewares: Vec<String>,
  pub routes: Vec<Route>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      host: IpAddr::V4("127.0.0.1".parse::<Ipv4Addr>().expect("invalid loopback")),
      port: 8080,
      middlewares: vec![],
      routes: Default::default(),
    }
  }
}

impl Config {
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    if !path.as_ref().exists() {
      return Err(Error::new(
        ErrorKind::IO,
        Some(format!("{} does not exist", path.as_ref().display())),
        None,
      ));
    }
    let (fmt, path) = match find_fmt(path.as_ref()) {
      Some((fmt, path)) => match path.exists() {
        true => (fmt, path),
        false => {
          return Err(Error::new(
            ErrorKind::IO,
            Some(format!("{}: file does not exist", path.display())),
            None,
          ))
        }
      },
      None => {
        return Err(Error::new(
          ErrorKind::IO,
          Some(format!(
            "{}: unknown config format",
            path.as_ref().display()
          )),
          None,
        ))
      }
    };
    (fmt.deserialize)(&path)
  }

  pub fn save<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    let formats = config_formats();
    let fmt = match formats.first() {
      Some(fmt) => fmt,
      None => {
        return Err(Error::new(
          ErrorKind::IO,
          Some(format!("unknown config format {}", path.as_ref().display())),
          None,
        ))
      }
    };
    (fmt.serialize)(path.as_ref(), self)
  }
}
