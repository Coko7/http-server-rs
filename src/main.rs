use anyhow::Result;
use log::LevelFilter;

use http_request::HttpRequest;
use http_response::HttpResponse;
use web_server::WebServer;

mod http;
mod http_request;
mod http_response;
mod web_server;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let server = WebServer::new("127.0.0.1:7878")?
        .route("GET /hello", get_hello)?
        .route("GET /mirror", get_mirror)?;

    server.run()
}

fn get_hello(request: &HttpRequest) -> Result<HttpResponse> {
    let name = request.query.get("name").map_or("World", |v| v);
    let body = format!("Hello {}!", name);

    HttpResponse::new().set_html_body(&body)
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponse::new().set_json_body(request)
}
