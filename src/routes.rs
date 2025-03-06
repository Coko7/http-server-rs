use anyhow::Result;
use std::fs;

use crate::http_response_builder::HttpResponseBuilder;
use crate::{http_request::HttpRequest, http_response::HttpResponse};

pub fn get_hello(request: &HttpRequest) -> Result<HttpResponse> {
    let name = request.query.get("name").map_or("World", |v| v);
    let body = format!("Hello {}!", name);

    HttpResponseBuilder::new()?
        .set_status("200 OK")?
        .set_html_body(&body)?
        .build()
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponseBuilder::new()?
        .set_status("200 OK")?
        .set_json_body(request)?
        .build()
}

pub fn get_404(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = fs::read_to_string("pages/404.html")?;

    HttpResponseBuilder::new()?
        .set_status("404 NOT FOUND")?
        .set_html_body(&body)?
        .build()
}
