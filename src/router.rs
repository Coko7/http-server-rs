use anyhow::{bail, Context, Result};
use log::{debug, trace};
use std::{collections::HashMap, fs, str::FromStr};

use crate::{
    file_server::FileServer,
    http::{
        response_status_codes::HttpStatusCode, HttpMethod, HttpRequest, HttpResponse,
        HttpResponseBuilder,
    },
};

#[derive(Debug)]
pub struct Router {
    pub routes: HashMap<StoredRoute, RoutingCallback>,
    pub catcher_routes: HashMap<HttpMethod, RoutingCallback>,
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
            catcher_routes: HashMap::new(),
            file_server: None,
        }
    }

    pub fn set_file_server(mut self, file_server: FileServer) -> Self {
        self.file_server = Some(file_server);
        self
    }

    fn find_matching_route(&self, request_route: &RequestRoute) -> Result<Option<&StoredRoute>> {
        let mut excluded: Vec<&StoredRoute> = vec![];
        let request_route_parts = request_route.path.split('/');
        trace!("trying to match request parts: {:?}", request_route_parts);

        let matching_candidates: Vec<_> = self
            .routes
            .keys()
            .filter(|route| route.method == request_route.method)
            .collect();

        for (idx, part) in request_route_parts.enumerate() {
            for match_candidate in matching_candidates.iter() {
                if excluded.contains(match_candidate) {
                    continue;
                };

                if let Some(match_part) = match_candidate.parts.get(idx) {
                    if !match_part.is_dynamic && !match_part.name.eq(part) {
                        trace!(
                            "excluding server route from search because part differ and not dynamic: {:?}",
                            match_candidate
                        );
                        excluded.push(match_candidate);
                    }
                } else {
                    trace!(
                        "excluding server route from search because too small: {:?}",
                        match_candidate
                    );
                    excluded.push(match_candidate);
                };
            }
        }

        let selected_routes: Vec<_> = matching_candidates
            .iter()
            .filter(|route| !excluded.contains(route))
            .collect();

        trace!(
            "selected routes (should only have 1 or 0): {:?}",
            selected_routes
        );

        match selected_routes.len() {
            0 => Ok(None),
            1 => Ok(Some(selected_routes.first().unwrap())),
            _ => bail!(
                "multiple selected routes even though that should not happen: {:?}",
                selected_routes
            ),
        }
    }

    pub fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let route_def = format!("{} {}", request.method, request.url);
        let route = RequestRoute::from_str(&route_def)?;
        debug!("trying to match route: {route_def}");

        // test against declared routes
        let matching_result = self.find_matching_route(&route)?;
        if let Some(matching_route) = matching_result {
            debug!("found matching server route: {:?}", matching_route);
            let routing_data = matching_route.extract_routing_data(&request.url)?;
            let callback = self
                .routes
                .get(matching_route)
                .context("failed to get callback, even though route should be a valid key")?;

            return callback(request, &routing_data);
        }

        debug!("no matching server route, trying other options...");

        // test against file server static mappings
        if let Some(file_server) = &self.file_server {
            debug!("attempting with file server");
            match file_server.handle_file_access(&route.path) {
                Ok(file_path) => {
                    let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
                    let content = fs::read(file_path)?;

                    return HttpResponseBuilder::new()
                        .set_raw_body(content)
                        .set_content_type(mime_type.as_ref())
                        .build();
                }
                Err(e) => debug!("no match with file server: {e}"),
            }
        }

        // test against catcher routes
        if let Some(catcher) = self.catcher_routes.get(&request.method) {
            debug!("defaulting to catcher for {}", request.method.to_string());
            return catcher(request, &RoutingData::default());
        }

        debug!("no default catcher, return 404");
        HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .build()
    }

    pub fn add_catcher_route(
        &mut self,
        method: HttpMethod,
        callback: RoutingCallback,
    ) -> Result<()> {
        if self.catcher_routes.contains_key(&method) {
            bail!(
                "cannot register catcher because one already exists for: {}",
                method.to_string()
            );
        }

        self.catcher_routes.insert(method, callback);
        Ok(())
    }

    pub fn add_route(
        &mut self,
        method: HttpMethod,
        path: &str,
        callback: RoutingCallback,
    ) -> Result<()> {
        let route = StoredRoute::new(method, path)?;

        if self.routes.contains_key(&route) {
            bail!(
                "cannot register route {:?} because a similar route already exists",
                route
            );
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

    pub fn catch_all(mut self, method: HttpMethod, callback: RoutingCallback) -> Result<Self> {
        self.add_catcher_route(method, callback)?;
        Ok(self)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct StoredRoute {
    pub method: HttpMethod,
    pub path: String,
    pub parts: Vec<RoutePart>,
}

impl StoredRoute {
    pub fn new(method: HttpMethod, path: &str) -> Result<Self> {
        let path = path.trim_matches('/').to_owned();

        let mut parts = vec![];
        for part in path.split('/') {
            let is_dynamic = part.starts_with(':');
            let value = if is_dynamic {
                part[1..].to_string()
            } else {
                part.to_string()
            };

            if value.contains(':') {
                bail!("nested `:` is not allowed in dynamic route part");
            }

            parts.push(RoutePart {
                is_dynamic,
                name: value,
            });
        }

        Ok(Self {
            method,
            path,
            parts,
        })
    }

    pub fn extract_routing_data(&self, request_url: &str) -> Result<RoutingData> {
        let request_parts: Vec<_> = request_url.split('/').filter(|p| !p.is_empty()).collect();

        let mut params: HashMap<String, Option<String>> = HashMap::new();
        for (idx, part) in self.parts.iter().enumerate() {
            if !part.is_dynamic {
                continue;
            }

            let value = request_parts.get(idx).map(|&value| value.to_owned());
            params.insert(part.name.to_owned(), value);
        }

        Ok(RoutingData { params })
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RoutePart {
    pub is_dynamic: bool,
    pub name: String,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RequestRoute {
    pub method: HttpMethod,
    pub path: String,
}

impl RequestRoute {
    pub fn new(method: HttpMethod, path: &str) -> RequestRoute {
        let path = path.trim_matches('/').to_owned();
        RequestRoute { method, path }
    }
}

impl FromStr for RequestRoute {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (method, path) = s
            .split_once(" ")
            .context("route should have following format: METHOD PATH (ex: GET /index)")?;
        let method = HttpMethod::from_str(method)?;

        Ok(RequestRoute::new(method, path))
    }
}

type RoutingCallback = fn(&HttpRequest, &RoutingData) -> Result<HttpResponse>;

#[derive(Debug, Default)]
pub struct RoutingData {
    params: HashMap<String, Option<String>>,
}

impl RoutingData {
    pub fn get_str_value(&self, param_name: &str) -> Result<Option<String>> {
        if let Some(param_value) = self.params.get(param_name) {
            Ok(param_value.to_owned())
        } else {
            bail!("no such route parameter: {param_name}")
        }
    }

    pub fn get_value<T: FromStr>(&self, param_name: &str) -> Result<Option<T>> {
        match self.get_str_value(param_name)? {
            Some(str_value) => match str_value.parse::<T>() {
                Ok(value) => Ok(Some(value)),
                Err(_) => bail!("failed to parse value `{}` for: {}", str_value, param_name),
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::http::{HttpRequestRaw, HttpResponseBuilder};

    use super::*;

    fn catcher_get_404(
        _request: &HttpRequest,
        _routing_data: &RoutingData,
    ) -> Result<HttpResponse> {
        HttpResponseBuilder::new()
            .set_html_body("404 YOU ARE LOST")
            .build()
    }

    fn get_hello_callback(
        _request: &HttpRequest,
        _routing_data: &RoutingData,
    ) -> Result<HttpResponse> {
        HttpResponseBuilder::new()
            .set_html_body("Hello World!")
            .build()
    }

    fn post_hello_callback(
        _request: &HttpRequest,
        _routing_data: &RoutingData,
    ) -> Result<HttpResponse> {
        HttpResponseBuilder::new()
            .set_html_body("Hello World from POST!")
            .build()
    }

    fn get_user_by_id(_request: &HttpRequest, routing_data: &RoutingData) -> Result<HttpResponse> {
        if let Some(id) = routing_data.get_value::<u32>("id").unwrap() {
            let username = format!("user_{id}");
            let json = json!({ "id": id, "username": username });

            HttpResponseBuilder::new().set_json_body(&json)?.build()
        } else {
            HttpResponseBuilder::new()
                .set_status(HttpStatusCode::BadRequest)
                .build()
        }
    }

    fn get_user_info(_request: &HttpRequest, routing_data: &RoutingData) -> Result<HttpResponse> {
        let id = routing_data
            .get_str_value("id")
            .unwrap()
            .unwrap_or(String::new());

        let info_field = routing_data
            .get_str_value("field")
            .unwrap()
            .unwrap_or(String::new());

        let username = format!("user_{id}");
        let json = json!({ "username": username, "field": info_field });

        HttpResponseBuilder::new().set_json_body(&json)?.build()
    }

    fn post_user_callback(
        _request: &HttpRequest,
        _routing_data: &RoutingData,
    ) -> Result<HttpResponse> {
        let json = json!({ "created": true });
        HttpResponseBuilder::new().set_json_body(&json)?.build()
    }

    #[test]
    fn test_unmatched_no_catcher() {
        let router = Router::new();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /hello HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!(HttpStatusCode::NotFound.to_string(), response.status);
    }

    #[test]
    fn test_unmatched_get_catcher() {
        let router = Router::new()
            .catch_all(HttpMethod::GET, catcher_get_404)
            .unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /not-a-real-page HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!("404 YOU ARE LOST\r\n".as_bytes(), response.body);
    }

    #[test]
    fn test_get_hello_html() {
        let router = Router::new()
            .get("/hello", get_hello_callback)
            .unwrap()
            .catch_all(HttpMethod::GET, catcher_get_404)
            .unwrap();

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
    fn test_get_hello_html_when_similar_route() {
        let router = Router::new()
            .get("/hello", get_hello_callback)
            .unwrap()
            .post("/hello", post_hello_callback)
            .unwrap()
            .catch_all(HttpMethod::GET, catcher_get_404)
            .unwrap();

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

    #[test]
    fn test_dynamic_route() {
        let router = Router::new()
            .get("/users/:id/details", get_user_by_id)
            .unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /users/5/details HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        let actual_res: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!("user_5", actual_res["username"]);
    }

    #[test]
    fn test_dynamic_route_value_parse() {
        let router = Router::new()
            .get("/users/:id/details", get_user_by_id)
            .unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /users/7/details HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        let actual_res: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(7, actual_res["id"]);
    }

    #[test]
    fn test_dynamic_route_no_value() {
        let router = Router::new().get("/users/:id", get_user_by_id).unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /users HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        assert_eq!(HttpStatusCode::BadRequest.to_string(), response.status);
    }

    #[test]
    fn test_dynamic_route_multiparams() {
        let router = Router::new()
            .get("/users/:id/info/:field", get_user_info)
            .unwrap();

        let request = HttpRequest::from_raw_request(HttpRequestRaw {
            request_line: "GET /users/17/info/gender HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: vec![],
        })
        .unwrap();

        let response = router.handle_request(&request).unwrap();
        let actual_res: Value = serde_json::from_slice(&response.body).unwrap();
        let expected_result = json!({ "username": "user_17", "field": "gender"});
        assert_eq!(expected_result, actual_res);
    }
}
