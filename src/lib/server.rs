use std::{
  collections::VecDeque,
  io::{stdout, Read, Write},
  net::{IpAddr, Shutdown, TcpListener, TcpStream},
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use log::{debug, error, info};

use crate::{Buffer, Config, Middleware, Middlewares, Request, Response, Router, Table};

#[derive(Default)]
pub struct Server {
  config: Config,
  router: Arc<Router>,
  middlewares: Vec<Arc<Mutex<dyn Middleware>>>,
}

impl Server {
  pub fn new(config: Config) -> Self {
    Self {
      config: config.clone(),
      router: Arc::new(Router::default().with_routes(config.routes)),
      middlewares: Vec::new(),
    }
  }

  pub fn with_middleware<M: Middleware + 'static>(mut self, m: M) -> Self {
    self.config.middlewares.push(m.name().clone());
    self.middlewares.push(Arc::new(Mutex::new(m)));
    self
  }

  pub fn with_middlewares<M: Middleware + 'static, I: IntoIterator<Item = M>>(
    mut self,
    ms: I,
  ) -> Self {
    for m in ms.into_iter() {
      self = self.with_middleware(m);
    }
    self
  }

  pub fn with_config(mut self, c: Config) -> Self {
    self.config = c;
    self
  }

  pub fn banner<W: Write>(&self, mut w: W) -> crate::Result<()> {
    writeln!(
      w,
      "🚀 Server running at \x1b[4m{}://{}:{}\x1b[0m\n",
      "http", self.config.host, self.config.port
    )?;
    writeln!(
      w,
      "🚗 \x1b[1;4mRoutes\x1b[0m{}\n",
      match self.config.routes.len() {
        0 => String::new(),
        n => format!(" ({})", n),
      }
    )?;
    let mut routes = Table::new().with_line_prefix("  📍 ").with_separator(" │ ");
    for route in &self.config.routes {
      routes.push([
        route
          .methods()
          .iter()
          .map(|m| format!("{}", m))
          .collect::<Vec<_>>()
          .join(", "),
        route.endpoint().clone(),
        route.kind_str().to_string(),
      ]);
    }
    routes.aligned().write(&mut w)?;
    writeln!(w)?;
    Ok(())
  }

  pub fn listen(mut self) -> crate::Result<()> {
    self = self.init_middlewares()?;
    self.banner(stdout())?;
    let listener = TcpListener::bind(format!("{}:{}", self.config.host, self.config.port)).unwrap();
    let mut handles = VecDeque::new();
    for stream in listener.incoming() {
      let mut stream = stream.unwrap();
      let middlewares = self.middlewares.clone();
      let router = self.router.clone();
      handles.push_back(thread::spawn(move || {
        if let Err(e) = Self::handle_request(&mut stream, &router, &middlewares) {
          error!("Handler crashed: {}", &e);
          let res: Response = e.into();
          if let Err(we) = res.write_to(&stream) {
            error!("Failed to write response: {}", we);
          }
        }
      }));
    }
    while let Some(handle) = handles.pop_front() {
      let _ = handle.join();
    }
    Ok(())
  }

  fn execute_middleware(
    request: &Request,
    mut response: Response,
    middleware: &Arc<Mutex<dyn Middleware>>,
  ) -> crate::Result<Response> {
    let mut m = None;
    loop {
      match middleware.try_lock() {
        Ok(g) => {
          debug!("Executing middleware: {}", g.name());
          m = Some(g);
          break;
        }
        Err(e) => {
          error!("Failed to lock middleware: {}", e);
          thread::sleep(Duration::from_millis(10));
        }
      }
    }
    response = m.unwrap().execute(request, response)?;
    Ok(response)
  }

  fn handle_request(
    mut stream: &TcpStream,
    router: &Router,
    middlewares: &Vec<Arc<Mutex<dyn Middleware>>>,
  ) -> crate::Result<Response> {
    info!("Connection accepted from '{}'", stream.peer_addr()?);
    let req = Request::from_reader(stream)?;
    let mut res = Response::default();
    for middleware in middlewares {
      res = Self::execute_middleware(&req, res, middleware)?;
    }
    res = router.dispatch(&req, res)?;
    let mut buf = vec![];
    res.write_to(&mut buf)?;
    debug!(
      "Response: {}",
      unsafe { std::str::from_utf8_unchecked(&buf) }.trim()
    );
    stream.write(&buf)?;
    stream.flush()?;
    stream.shutdown(Shutdown::Both)?;
    Ok(res)
  }

  fn init_middlewares(mut self) -> crate::Result<Self> {
    #[cfg(feature = "cors")]
    Middlewares::register(String::from(crate::cors::CORS_MW_NAME), || {
      Ok(Arc::new(Mutex::new(crate::cors::CorsMiddleware::new())))
    });
    for mw_name in &self.config.middlewares {
      let found = self.middlewares.iter().find(|mw| {
        let g = mw.lock().expect("failed to lock middleware");
        if g.name().eq_ignore_ascii_case(&mw_name) {
          return true;
        }
        return false;
      });
      if found.is_none() {
        self.middlewares.push(Middlewares::create(&mw_name)?)
      }
    }
    Ok(self)
  }
}
