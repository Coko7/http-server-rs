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
- [ ] MIME support 🎭
    - [x] support for file download (`HttpResponse.body` is now `Vec<u8>`)
    - [x] support for file upload (`HttpRequest.body` is now `Vec<u8>`)
- [ ] HTTPS 🛡️
- [ ] Improved routing 🚄 (W.I.P)
    - [ ] support for dynamic paths: `/foo/{:id}/bar`

## Usage example

```rs
use anyhow::Result;
use http_server::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use http_server::web_server::WebServer;
use std::fs;

fn main() -> Result<()> {
    let router = Router::new()
        // get routes
        .get("/", routes::get_hello)?
        .get("/hello", routes::get_hello)?
        .get("/mirror", routes::get_mirror)?
        // post route
        .post("/paste", routes::post_paste_data)?
        // 404 Catch all
        .get("/*", routes::get_404)?;

    let server = WebServer::new("127.0.0.1:7878", router)?;
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

pub fn post_paste_data(request: &HttpRequest) -> Result<HttpResponse> {
    let body = request.body.clone().unwrap_or(String::new());
    if body.len() > 1_000_000 {
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::BadRequest)
            .set_json_body(
                &json!({"status": "400 Bad Request", "message": "too many characters!"}),
            )?
            .build();
    }

    // Do stuff...

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::OK)
        .set_json_body(&json!({"status": "200 OK", "message": "data has been saved"}))?
        .build()
}

pub fn get_404(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = fs::read_to_string("pages/404.html")?;

    HttpResponseBuilder::new()
        .set_status("404 NOT FOUND")
        .set_html_body(&body)
        .build()
}
```
