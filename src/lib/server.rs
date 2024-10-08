use std::{
  collections::VecDeque,
  io::Write as _,
  net::{IpAddr, Shutdown, TcpListener, TcpStream},
  sync::Arc,
  thread,
};

use crate::Response;

pub trait Middleware: Send + Sync {
  fn execute(&mut self, response: &mut Response) -> crate::Result<()>;
}

pub trait CorsMiddleware

pub struct Server {
  listen_addr: IpAddr,
  port: u16,
  middlewares: Vec<Arc<dyn Middleware>>,
}

impl Server {
  pub fn new(listen_addr: IpAddr, port: u16) -> Self {
    Self {
      listen_addr,
      port,
      middlewares: Vec::new(),
    }
  }

  pub fn listen(self) -> crate::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", self.listen_addr, self.port)).unwrap();

    let mut handles = VecDeque::new();
    for stream in listener.incoming() {
      let stream = stream.unwrap();
      handles.push_back(thread::spawn(move || {
        Self::handle_request(stream, &self.middlewares)
      }));
    }
    while let Some(handle) = handles.pop_front() {
      let _ = handle.join();
    }
    Ok(())
  }

  fn handle_request(
    mut stream: TcpStream,
    middlewares: &Vec<Arc<dyn Fn(&mut Response) -> crate::Result<()>>>,
  ) -> crate::Result<Response> {
    println!("Connection accepted from '{}'", stream.peer_addr()?);
    let mut res = Response::default();
    for middleware in middlewares {
      middleware(&mut res)?;
    }
    res.write_to(&stream)?;
    stream.flush()?;
    stream.shutdown(Shutdown::Both)?;
    Ok(res)
  }
}
