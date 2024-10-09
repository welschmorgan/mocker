use std::{
  any::Any,
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use log::debug;

use crate::{Error, ErrorKind, Method, Request, Response, Route, RouteKind, Status, Store, Value};

pub trait RouteHandler {
  fn handle(&self, req: &Request, res: Response) -> crate::Result<Response>;
}

pub struct StoreRouteHandler {
  route: Route,
  store: Mutex<Store>,
}

impl StoreRouteHandler {
  pub fn new<P: AsRef<Path>, I: AsRef<str>>(route: Route, path: P, identifier: I) -> Self {
    Self {
      route,
      store: Mutex::new(Store::json(path, identifier)),
    }
  }

  pub fn load_entity(&self, req: &Request) -> crate::Result<Response> {
    let mut store = self.store.lock()?;
    let (id_key, id_value) = match req.query_param(store.identifier()) {
      Some((key, Some(val))) => (key.clone(), Value::from(val.clone())),
      Some((key, None)) => {
        return Ok(Response::default().with_status_code(400).with_body(format!(
          "Identifier '{}' was found in query params but has no value",
          store.identifier()
        )))
      }
      None => {
        return Ok(Response::default().with_status_code(400).with_body(format!(
          "Identifier '{}' not found in query params",
          store.identifier()
        )))
      }
    };
    store.load()?;
    match store.find(&id_value) {
      Some(obj) => Response::api(Status::OK, obj),
      None => Ok(Response::default().with_status_code(404).with_body(format!(
        "Entity with `{}` = {} was not found",
        id_key, id_value
      ))),
    }
  }

  pub fn create_entity(&self, req: &Request) -> crate::Result<Response> {
    let mut store = self.store.lock()?;
    store.load()?;
    let new_data = req.parse_body::<HashMap<String, Value>>()?;
    let id = match store.id_field(&new_data) {
      Some((_key, value)) => value.clone(),
      None => Value::Null,
    };
    store.create(new_data)?;
    store.save()?;
    return Response::api(Status::Created, &id);
  }
}

impl RouteHandler for StoreRouteHandler {
  fn handle(&self, req: &Request, res: Response) -> crate::Result<Response> {
    match req.method().expect("Missing method") {
      Method::Get => self.load_entity(req),
      Method::Post => self.create_entity(req),
      Method::Put => {
        todo!("StoreRouteHandler PUT method");
      }
      Method::Patch => {
        todo!("StoreRouteHandler PATCH method");
      }
      Method::Delete => {
        todo!("StoreRouteHandler DELETE method");
      }
      m => Err(Error::new(
        ErrorKind::Unknown,
        Some(format!("unsupported method: {:?}", m)),
        None,
      )),
    }
  }
}

#[cfg(feature = "js")]
pub struct ScriptRouteHandler {
  route: Route,
  script_path: PathBuf,
  func_name: String,
}

#[cfg(feature = "js")]
impl ScriptRouteHandler {
  pub fn new<S: AsRef<Path>, F: AsRef<str>>(route: Route, script_path: S, func_name: F) -> Self {
    Self {
      route,
      script_path: script_path.as_ref().to_path_buf(),
      func_name: func_name.as_ref().to_string(),
    }
  }
}

#[cfg(feature = "js")]
impl RouteHandler for ScriptRouteHandler {
  fn handle(&self, req: &Request, res: Response) -> crate::Result<Response> {
    todo!();
    Ok(res)
  }
}

#[derive(Default, Clone)]
pub struct Router(HashMap<String, HashMap<Method, Arc<dyn RouteHandler>>>);

unsafe impl Send for Router {}
unsafe impl Sync for Router {}

impl Router {
  pub fn set<M: IntoIterator<Item = Method>, E: AsRef<str>, H: RouteHandler + 'static>(
    &mut self,
    methods: M,
    endpoint: E,
    handler: H,
  ) {
    let entry = self
      .0
      .entry(endpoint.as_ref().to_string())
      .or_insert_with(|| HashMap::new());
    let handler = Arc::new(handler);
    for meth in methods.into_iter() {
      entry.insert(meth, handler.clone());
    }
  }

  pub fn handler<E: AsRef<str>>(
    &self,
    method: Method,
    endpoint: E,
  ) -> Option<&Arc<dyn RouteHandler>> {
    match self
      .0
      .iter()
      .find(|(_endpoint, _methods)| _endpoint.as_str().eq(endpoint.as_ref()))
    {
      Some((_endpoint, methods)) => match methods.iter().find(|(m, h)| method as u8 == **m as u8) {
        Some((m, h)) => Some(h),
        None => None,
      },
      None => None,
    }
  }

  pub fn dispatch(&self, req: &Request, res: Response) -> crate::Result<Response> {
    let endpoint = req.path().unwrap_or_else(|| "/");
    match self.handler(req.method().unwrap_or_else(|| Method::Get), endpoint) {
      Some(handler) => {
        debug!("Found handler for '{}'", endpoint);
        handler.handle(req, res)
      }
      None => Ok(Response::default().with_status_code(404)),
    }
  }

  pub fn with_routes<I: IntoIterator<Item = crate::Route>>(mut self, routes: I) -> Self {
    for route in routes.into_iter() {
      match route.kind() {
        #[cfg(feature = "js")]
        RouteKind::Script { script, func } => self.set(
          route.methods().clone(),
          route.endpoint(),
          ScriptRouteHandler::new(route.clone(), script, func),
        ),
        #[cfg(feature = "json")]
        RouteKind::Store { path, identifier } => self.set(
          route.methods().clone(),
          route.endpoint(),
          StoreRouteHandler::new(route.clone(), path, identifier),
        ),
      }
    }
    self
  }
}
