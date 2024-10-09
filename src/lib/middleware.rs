use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use lazy_static::lazy_static;

use crate::{Error, ErrorKind, Method, Request, Response};

pub trait Middleware: Send + Sync {
  fn name(&self) -> &String;
  fn supported_methods(&self) -> Vec<Method>;
  fn execute(&mut self, request: &Request, response: Response) -> crate::Result<Response>;
}

pub struct Middlewares(HashMap<String, Arc<dyn Fn() -> crate::Result<Arc<Mutex<dyn Middleware>>>>>);

unsafe impl Send for Middlewares {}
unsafe impl Sync for Middlewares {}

impl Middlewares {
  pub fn create<N: AsRef<str>>(name: N) -> crate::Result<Arc<Mutex<dyn Middleware>>> {
    match Self::constructor(name.as_ref()) {
      Some(ctor) => ctor(),
      None => Err(Error::new(
        ErrorKind::Unknown,
        Some(format!("unknown middleware '{}'", name.as_ref())),
        None,
      )),
    }
  }

  pub fn constructor<N: AsRef<str>>(
    name: N,
  ) -> Option<Arc<dyn Fn() -> crate::Result<Arc<Mutex<dyn Middleware>>>>> {
    let g = middlewares.lock().unwrap();
    match g
      .0
      .iter()
      .find(|(k, v)| k.eq_ignore_ascii_case(name.as_ref()))
    {
      Some((name, constructor)) => Some(constructor.clone()),
      None => None,
    }
  }

  pub fn register<N: AsRef<str>, M: Fn() -> crate::Result<Arc<Mutex<dyn Middleware>>> + 'static>(
    name: N,
    ctor: M,
  ) {
    let mut g = middlewares.lock().unwrap();
    g.0.insert(name.as_ref().to_string(), Arc::new(ctor));
  }
}

lazy_static! {
  static ref middlewares: Arc<Mutex<Middlewares>> =
    Arc::new(Mutex::new(Middlewares(HashMap::new())));
}
