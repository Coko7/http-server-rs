use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use std::{
    collections::HashMap,
    io::Write,
    net::{TcpListener, TcpStream},
    str::FromStr,
};

use crate::{
    http::{HttpMethod, HttpRequest, HttpResponse, HttpVersion},
    thread_pool::ThreadPool,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Route {
    pub verb: HttpMethod,
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

        Ok(Route { verb, path })
    }
}

pub struct WebServer {
    pub hostname: String,
    pub routes: HashMap<Route, fn(&HttpRequest) -> Result<HttpResponse>>,
    version: HttpVersion,
    listener: TcpListener,
    pool: ThreadPool,
}

impl WebServer {
    pub fn new(hostname: &str) -> Result<Self> {
        let listener = TcpListener::bind(hostname).unwrap();
        let pool = ThreadPool::new(4);

        Ok(WebServer {
            hostname: hostname.to_string(),
            routes: HashMap::new(),
            version: HttpVersion::HTTP1_1,
            listener,
            pool,
        })
    }

    pub fn run(&self) -> Result<()> {
        info!("server started on {}", self.hostname);
        info!("awaiting connections...");

        for stream in self.listener.incoming() {
            debug!("{}", "new connection!");
            let stream = stream?;

            let routes = self.routes.clone();

            self.pool.execute(move || {
                let result = handle_connection(stream, &routes);
                if let Err(result) = result {
                    let error = format!("error: {}", result);
                    error!("{}", error);
                }
            });
        }

        Ok(())
    }

    pub fn http_version(mut self, version: HttpVersion) -> Self {
        self.version = version;
        self
    }

    pub fn route(
        mut self,
        route_def: &str,
        callback: fn(&HttpRequest) -> Result<HttpResponse>,
    ) -> Result<Self> {
        let route = Route::from_str(route_def)?;

        if self.routes.contains_key(&route) {
            return Err(anyhow!(
                "cannot register route {:?} because a similar route already exists",
                route
            ));
        }

        self.routes.insert(route, callback);
        Ok(self)
    }
}

fn handle_request(
    request: &HttpRequest,
    routes: &HashMap<Route, fn(&HttpRequest) -> Result<HttpResponse>>,
) -> Result<HttpResponse> {
    let route_def = format!("{} {}", request.method.to_string(), request.url);
    let route = Route::from_str(&route_def)?;
    debug!("route: {route_def}");

    let response = if let Some(route_callback) = routes.get(&route) {
        route_callback(request)
    } else {
        let catch_all_route = Route::from_str("GET /*")?;
        if let Some(catch_all_callback) = routes.get(&catch_all_route) {
            return catch_all_callback(request);
        }

        Err(anyhow!("unsupported route: {route_def}"))
    };

    response
}

fn handle_connection(
    mut stream: TcpStream,
    routes: &HashMap<Route, fn(&HttpRequest) -> Result<HttpResponse>>,
) -> Result<()> {
    let request = HttpRequest::from_tcp(&stream)?;

    let mut request_dbg = String::new();
    request_dbg.push_str(">>> Request START <<<\r\n");
    request_dbg.push_str(
        format!(
            "{} {} {}\r\n",
            request.method.to_string(),
            request.resource_path,
            request.version.to_string(),
        )
        .as_str(),
    );

    request_dbg.push_str(">>> HEADERS <<<\r\n");

    for header in request.headers.iter() {
        request_dbg.push_str(format!("{}: {}\r\n", header.name, header.value).as_str());
    }

    if let Some(ref body) = request.body {
        request_dbg.push_str(">>> BODY <<<\r\n");
        request_dbg.push_str(format!("{}\r\n", body).as_str());
    }

    request_dbg.push_str(">>> Request END <<<\r\n");
    debug!("{}", request_dbg);

    let response = handle_request(&request, &routes)?.to_string()?;

    stream.write_all(response.as_bytes())?;
    Ok(())
}
