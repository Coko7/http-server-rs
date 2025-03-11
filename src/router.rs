use anyhow::{anyhow, Context, Result};
use log::debug;
use std::{collections::HashMap, str::FromStr};

use crate::http::{HttpMethod, HttpRequest, HttpResponse};

type RoutingCallback = fn(&HttpRequest) -> Result<HttpResponse>;

#[derive(Debug)]
pub struct Router {
    pub routes: HashMap<Route, RoutingCallback>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let route_def = format!("{} {}", request.method.to_string(), request.url);
        let route = Route::from_str(&route_def)?;
        debug!("route: {route_def}");

        let response = if let Some(route_callback) = self.routes.get(&route) {
            route_callback(request)
        } else {
            let catch_all_route = Route::from_str("GET /*")?;
            if let Some(catch_all_callback) = self.routes.get(&catch_all_route) {
                return catch_all_callback(request);
            }

            Err(anyhow!("unsupported route: {route_def}"))
        };

        response
    }

    pub fn add_route(
        &mut self,
        method: HttpMethod,
        path: &str,
        callback: RoutingCallback,
    ) -> Result<()> {
        let path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        let route = Route {
            method,
            path: path.to_string(),
        };

        if self.routes.contains_key(&route) {
            return Err(anyhow!(
                "cannot register route {:?} because a similar route already exists",
                route
            ));
        }

        self.routes.insert(route, callback);
        Ok(())
    }

    pub fn get(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::GET, path, callback)?;
        Ok(self)
    }

    pub fn head(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::HEAD, path, callback)?;
        Ok(self)
    }

    pub fn post(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::POST, path, callback)?;
        Ok(self)
    }

    pub fn put(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::PUT, path, callback)?;
        Ok(self)
    }

    pub fn delete(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::DELETE, path, callback)?;
        Ok(self)
    }

    pub fn connect(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::CONNECT, path, callback)?;
        Ok(self)
    }

    pub fn options(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::OPTIONS, path, callback)?;
        Ok(self)
    }

    pub fn trace(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::TRACE, path, callback)?;
        Ok(self)
    }

    pub fn patch(mut self, path: &str, callback: RoutingCallback) -> Result<Self> {
        self.add_route(HttpMethod::PATCH, path, callback)?;
        Ok(self)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
}

impl FromStr for Route {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (verb, path) = s.split_once(" ").context("route should have: VERB PATH")?;
        let verb = HttpMethod::from_str(verb)?;

        let path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        Ok(Route { method: verb, path })
    }
}

#[cfg(test)]
mod tests {
    use crate::http::{HttpRequestRaw, HttpResponseBuilder};

    use super::*;

    fn hello_callback(request: &HttpRequest) -> Result<HttpResponse> {
        HttpResponseBuilder::new()
            .set_html_body("Hello World!")
            .build()
    }

    #[test]
    fn test_get_hello() {
        let router = Router::new().get("/hello", hello_callback).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /hello HTTP/1.1".to_string(),
            headers: Vec::new(),
            body: None,
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("Hello World!\r\n", response.body);
    }

    #[test]
    fn test_unknown_route_err() {
        let router = Router::new();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /hello HTTP/1.1".to_string(),
            headers: Vec::new(),
            body: None,
        })
        .unwrap();

        let response = router.handle_request(&request);
        assert!(response.is_err());
    }

    #[test]
    fn test_unknown_has_fallback() {
        let router = Router::new().get("/*", hello_callback).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /not-a-real-page HTTP/1.1".to_string(),
            headers: Vec::new(),
            body: None,
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("Hello World!\r\n", response.body);
    }
}
