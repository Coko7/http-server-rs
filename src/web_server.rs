use anyhow::Result;
use log::{error, info, trace};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::{
    http::{HttpRequest, HttpVersion},
    router::Router,
    thread_pool::ThreadPool,
};

pub struct WebServer {
    pub hostname: String,
    pub router: Arc<Mutex<Router>>,
    version: HttpVersion,
    listener: TcpListener,
    pool: ThreadPool,
}

impl WebServer {
    pub fn new(hostname: &str, router: Router) -> Result<Self> {
        let listener = TcpListener::bind(hostname).unwrap();
        let pool = ThreadPool::new(4);

        Ok(WebServer {
            hostname: hostname.to_owned(),
            router: Arc::new(Mutex::new(router)),
            version: HttpVersion::HTTP1_1,
            listener,
            pool,
        })
    }

    pub fn run(&self) -> Result<()> {
        info!("server started on {}", self.hostname);
        info!("awaiting connections...");

        for stream in self.listener.incoming() {
            trace!("{}", "got new tcp connection!");
            let stream = stream?;

            let router_clone = Arc::clone(&self.router);
            self.pool.execute(move || {
                let result = handle_connection(router_clone, stream);
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
}

fn handle_connection(router: Arc<Mutex<Router>>, mut stream: TcpStream) -> Result<()> {
    let request = HttpRequest::from_tcp(&stream)?;

    let mut request_dbg = String::new();
    request_dbg.push_str("\r\n>>> Request START <<<\r\n");
    request_dbg.push_str(
        format!(
            "{} {} {}\r\n",
            request.method, request.resource_path, request.version,
        )
        .as_str(),
    );

    request_dbg.push_str(">>> HEADERS <<<\r\n");

    for header in request.headers.values() {
        request_dbg.push_str(format!("{}: {}\r\n", header.name, header.value).as_str());
    }

    if !request.body.is_empty() {
        request_dbg.push_str(">>> BODY <<<\r\n");
        match String::from_utf8(request.body.clone()) {
            Ok(value) => request_dbg.push_str(format!("::TEXT DATA::\r\n{}\r\n", value).as_str()),
            Err(e) => {
                trace!(
                    "failed to parse to UTF8 str -> likely got binary body: {}",
                    e
                );
                request_dbg.push_str("::BINARY DATA::\r\n");
            }
        }
    }

    request_dbg.push_str(">>> Request END <<<\r\n");
    trace!("{}", request_dbg);

    let response = router
        .lock()
        .unwrap()
        .handle_request(&request)?
        .to_bytes()?;

    stream.write_all(&response)?;
    Ok(())
}
