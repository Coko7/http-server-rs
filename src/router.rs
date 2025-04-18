use anyhow::{anyhow, Context, Result};
use log::{trace, warn};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::http::{
    response_status_codes::HttpStatusCode, HttpMethod, HttpRequest, HttpResponse,
    HttpResponseBuilder,
};

type RoutingCallback = fn(&HttpRequest) -> Result<HttpResponse>;
type StaticFilesMount = (String, String);

#[derive(Debug)]
pub struct Router {
    pub routes: HashMap<Route, RoutingCallback>,
    pub file_server: Option<StaticFilesMount>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            file_server: None,
        }
    }

    pub fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let route_def = format!("{} {}", request.method, request.url);
        let route = Route::from_str(&route_def)?;
        trace!("trying to match route: {route_def}");

        if let Some(file_server) = &self.file_server {
            let (route_prefix, dir_path) = file_server;
            if let Some(filename) = route.path.strip_prefix(route_prefix) {
                let safe_filename = match Path::new(filename).file_name() {
                    Some(filename) => Ok(filename.to_owned()),
                    None => Err(anyhow!("not a file: {filename}")),
                };

                if let Err(e) = safe_filename {
                    warn!("failed to serve file '{filename}': {e}");
                    return HttpResponseBuilder::new()
                        .set_status(HttpStatusCode::BadRequest)
                        .build();
                }

                let safe_filename = safe_filename.unwrap();
                let filepath = PathBuf::from(dir_path).join(safe_filename);
                let mime_type = mime_guess::from_path(&filepath).first_or_octet_stream();
                let content = fs::read(filepath)?;

                return HttpResponseBuilder::new()
                    .set_raw_body(content)
                    .set_content_type(mime_type.as_ref())
                    .build();
            }
        }

        let response = if let Some(route_callback) = self.routes.get(&route) {
            route_callback(request)
        } else {
            let catch_all_route = Route::from_str("GET /*")?;
            if let Some(catch_all_callback) = self.routes.get(&catch_all_route) {
                return catch_all_callback(request);
            }

            Err(anyhow!("failed to match route: {route_def}"))
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
            path.to_owned()
        } else {
            format!("{}/", path)
        };

        let route = Route {
            method,
            path: path.to_owned(),
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

    pub fn setup_file_serving(mut self, route: &str, directory_path: &str) -> Result<Self> {
        self.file_server = Some((route.to_owned(), directory_path.to_owned()));
        Ok(self)
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
            path.to_owned()
        } else {
            format!("{}/", path)
        };

        Ok(Route { method: verb, path })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::http::{HttpRequestRaw, HttpResponseBuilder};

    use super::*;

    fn get_hello_callback(_request: &HttpRequest) -> Result<HttpResponse> {
        HttpResponseBuilder::new()
            .set_html_body("Hello World!")
            .build()
    }

    fn post_user_callback(_request: &HttpRequest) -> Result<HttpResponse> {
        let json = json!({ "created": true });
        HttpResponseBuilder::new().set_json_body(&json)?.build()
    }

    #[test]
    fn test_unknown_route_err() {
        let router = Router::new();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /hello HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request);
        assert!(response.is_err());
    }

    #[test]
    fn test_unknown_has_fallback() {
        let router = Router::new().get("/*", get_hello_callback).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /not-a-real-page HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("Hello World!\r\n".as_bytes(), response.body);
    }

    #[test]
    fn test_get_hello_html() {
        let router = Router::new().get("/hello", get_hello_callback).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /hello HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("Hello World!\r\n".as_bytes(), response.body);
    }

    #[test]
    fn test_post_user_json() {
        let router = Router::new().post("/user", post_user_callback).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "POST /user HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("{\"created\":true}\r\n".as_bytes(), response.body);
    }
}
