use anyhow::Result;
use http_response::HttpResponse;
use log::LevelFilter;
use web_server::WebServer;

use http_request::HttpRequest;
use serde::Serialize;

mod http;
mod http_request;
mod http_response;
mod web_server;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let server = WebServer::new("127.0.0.1:7878")?
        .register("GET /hello", get_hello)?
        .register("GET /mirror", get_mirror)?;

    server.run()
}

fn get_hello(_request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponse::new().set_html_body("Hello world!")
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponse::new()
        .set_start_line("HTTP/1.1 200 OK")?
        .set_json_body(request)
}
