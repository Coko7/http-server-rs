use std::{
    collections::HashMap,
    io::Write,
    net::{TcpListener, TcpStream},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use log::{debug, error, info};

use crate::{http::HttpVerb, http_request::HttpRequest, http_response::HttpResponse};

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Route {
    pub verb: HttpVerb,
    pub path: String,
}

impl FromStr for Route {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (verb, path) = s.split_once(" ").context("route should have: VERB PATH")?;
        let verb = HttpVerb::from_str(verb)?;

        Ok(Route {
            verb,
            path: path.to_string(),
        })
    }
}

pub struct WebServer {
    pub hostname: String,
    pub routes: HashMap<Route, fn(&HttpRequest) -> Result<HttpResponse>>,
    listener: TcpListener,
}

impl WebServer {
    pub fn new(hostname: &str) -> Result<Self> {
        let listener = TcpListener::bind(hostname).unwrap();
        Ok(WebServer {
            hostname: hostname.to_string(),
            routes: HashMap::new(),
            listener,
        })
    }

    pub fn run(&self) -> Result<()> {
        info!("server started on {}", self.hostname);
        info!("awaiting connections...");

        for stream in self.listener.incoming() {
            debug!("{}", "new connection!".green());
            let stream = stream?;
            let result = self.handle_connection(stream);
            if let Err(result) = result {
                let error = format!("error: {}", result);
                error!("{}", error.red());
            }
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let request = HttpRequest::from_tcp(&stream)?;

        debug!("{}", ">>> Request START <<<".red());
        debug!(
            "{} {} {}",
            request.verb.to_string(),
            request.resource_path,
            request.version.to_string()
        );

        debug!("{}", ">>> HEADERS <<<".red());
        for (key, value) in request.headers.iter() {
            debug!("{}: {}", key, value);
        }

        if let Some(ref body) = request.body {
            debug!("{}", ">>> BODY <<<".red());
            debug!("{}", body);
        }

        debug!("{}", ">>> Request END <<<".red());

        let response = self.handle_request(&request)?.to_string();

        debug!("{}", "response sent!".blue());

        stream.write_all(response.as_bytes())?;
        Ok(())
    }

    fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let route_def = format!("{} {}", request.verb.to_string(), request.url);
        let route = Route::from_str(&route_def)?;
        debug!("route: {route_def}");

        let response = if let Some(route_callback) = self.routes.get(&route) {
            route_callback(request)
        } else {
            Err(anyhow!("unsupported route: {route_def}"))
        };

        response
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
