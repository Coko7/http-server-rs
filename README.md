# 🌐 http-server-rs

![build](https://github.com/coko7/http-server-rs/actions/workflows/rust.yml/badge.svg)

A simple http server from scratch using `std::net::tcp`.
This is for learning and playing purpose only.

Should never be used in production for obvious reasons 💀

## Roadmap

- [x] HTTP 1.0/1.1 support ✨
- [x] ~~HTTP 0.9 support 👴~~ *(support was temporarily dropped to simplify code, IT WILL COME BACK!)*
- [x] Routing 🚆
- [x] Multi-threading 🤹
- [x] Headers + cookies 🍪
- [ ] HTTPS 🛡️
- [ ] Improved routing 🚄

## Usage

```rs
use anyhow::Result;
use http_server::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use http_server::web_server::WebServer;
use std::fs;

fn main() -> Result<()> {
    let server = WebServer::new("127.0.0.1:7878")?
        .route("GET /hello", get_hello)?
        .route("GET /mirror", get_mirror)?
        .route("GET /*", get_404)?;

    server.run()
    Ok(())
}

pub fn get_hello(request: &HttpRequest) -> Result<HttpResponse> {
    let name = request.query.get("name").map_or("World", |v| v);
    let body = format!("Hello {}!", name);

    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponseBuilder::new().set_json_body(request)?.build()
}

pub fn get_404(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = fs::read_to_string("pages/404.html")?;

    HttpResponseBuilder::new()
        .set_status("404 NOT FOUND")
        .set_html_body(&body)
        .build()
}
```
