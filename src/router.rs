use anyhow::{anyhow, bail, Context, Result};
use log::{trace, warn};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::http::{HttpMethod, HttpRequest, HttpResponse, HttpResponseBuilder};

type RoutingCallback = fn(&HttpRequest) -> Result<HttpResponse>;

#[derive(Debug)]
pub struct FileServer {
    pub mount_points: HashSet<MountPoint>,
}

impl FileServer {
    pub fn new() -> Self {
        Self {
            mount_points: HashSet::new(),
        }
    }

    pub fn add_dir_mount(mut self, route: &str, dir_path: &str) -> Result<Self> {
        let mount_point = MountPoint {
            route: route.to_owned(),
            fs_path: PathBuf::from(dir_path),
            is_directory: true,
        };

        if !self.mount_points.insert(mount_point) {
            bail!("directory mount already exists");
        }

        Ok(self)
    }

    pub fn map_file(mut self, route: &str, file_path: &str) -> Result<Self> {
        let mount_point = MountPoint {
            route: route.to_owned(),
            fs_path: PathBuf::from(file_path),
            is_directory: false,
        };

        if !self.mount_points.insert(mount_point) {
            bail!("a similar file map already exists");
        }

        Ok(self)
    }

    fn get_file(file_path: PathBuf) -> Result<PathBuf> {
        if !file_path.exists() {
            bail!("file not found: {}", file_path.display());
        }

        if !file_path.is_file() {
            bail!("not a file: {}", file_path.display());
        }

        Ok(file_path)
    }

    pub fn handle_static_file_access(&self, file: &str) -> Result<PathBuf> {
        let file_path = self
            .mount_points
            .iter()
            .filter(|mp| !mp.is_directory)
            .find(|mp| mp.route == file)
            .map(|mp| mp.fs_path.clone());

        if let Some(file_path) = file_path {
            return FileServer::get_file(file_path);
        }

        let dir_mount_point = self
            .mount_points
            .iter()
            .filter(|mp| mp.is_directory)
            .find(|mp| file.starts_with(&mp.route));

        if let Some(dir_mount_point) = dir_mount_point {
            let file_name = file
                .strip_prefix(&dir_mount_point.route)
                .with_context(|| format!("file should have prefix: {}", dir_mount_point.route))?;

            let safe_file_name = match Path::new(file_name).file_name() {
                Some(filename) => Ok(filename.to_owned()),
                None => Err(anyhow!("invalid file name: {file}")),
            }?;

            let file_path = dir_mount_point.fs_path.join(safe_file_name);
            return Self::get_file(file_path);
        }

        bail!("failed to map file: {file}")
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MountPoint {
    pub route: String,
    pub fs_path: PathBuf,
    pub is_directory: bool,
}

#[derive(Debug)]
pub struct Router {
    pub routes: HashMap<Route, RoutingCallback>,
    pub file_server: Option<FileServer>,
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

    pub fn set_file_server(mut self, file_server: FileServer) -> Self {
        self.file_server = Some(file_server);
        self
    }

    pub fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let route_def = format!("{} {}", request.method, request.url);
        let route = Route::from_str(&route_def)?;
        trace!("trying to match route: {route_def}");

        if let Some(file_server) = &self.file_server {
            match file_server.handle_static_file_access(&route.path) {
                Ok(file_path) => {
                    let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
                    let content = fs::read(file_path)?;

                    return HttpResponseBuilder::new()
                        .set_raw_body(content)
                        .set_content_type(mime_type.as_ref())
                        .build();
                }
                Err(e) => warn!("failed to match file: {e}"),
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
