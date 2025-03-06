use log::LevelFilter;

use web_server::WebServer;

mod http;
mod http_request;
mod http_response;
mod http_response_builder;
mod routes;
mod thread_pool;
mod web_server;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let server = WebServer::new("127.0.0.1:7878")?
        .route("GET /hello", routes::get_hello)?
        .route("GET /mirror", routes::get_mirror)?
        .route("GET /*", routes::get_404)?;

    server.run()
}
