use lazy_static::lazy_static;

use crate::{Method, Middleware, Request, Response};

pub const CORS_MW_NAME: &'static str = "Cors";

pub struct CorsMiddleware {
  name: String,
}

impl CorsMiddleware {
  pub fn new() -> Self {
    Self {
      name: CORS_MW_NAME.to_string(),
    }
  }
}

impl Middleware for CorsMiddleware {
  fn name(&self) -> &String {
    &self.name
  }

  fn supported_methods(&self) -> Vec<Method> {
    return vec![Method::Options];
  }

  fn execute(&mut self, request: &Request, mut response: Response) -> crate::Result<Response> {
    response.set_header("Access-Control-Allow-Origin", "*");
    Ok(response)
  }
}
