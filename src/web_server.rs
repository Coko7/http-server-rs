use anyhow::Result;
use log::{debug, error, info};
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
    pub fn new(hostname: &str) -> Result<Self> {
        let listener = TcpListener::bind(hostname).unwrap();
        let pool = ThreadPool::new(4);

        Ok(WebServer {
            hostname: hostname.to_string(),
            router: Arc::new(Mutex::new(Router::new())),
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

    for header in request.headers.values() {
        request_dbg.push_str(format!("{}: {}\r\n", header.name, header.value).as_str());
    }

    if let Some(ref body) = request.body {
        request_dbg.push_str(">>> BODY <<<\r\n");
        request_dbg.push_str(format!("{}\r\n", body).as_str());
    }

    request_dbg.push_str(">>> Request END <<<\r\n");
    debug!("{}", request_dbg);

    let response = router
        .lock()
        .unwrap()
        .handle_request(&request)?
        .to_string()?;

    stream.write_all(response.as_bytes())?;
    Ok(())
}
