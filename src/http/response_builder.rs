use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::trace;
use serde::Serialize;

use super::{
    response_status_codes::HttpStatusCode, HttpCookie, HttpHeader, HttpResponse, HttpVersion,
};

pub struct HttpResponseBuilder {
    response: HttpResponse,
}

impl Default for HttpResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        HttpResponseBuilder {
            response: HttpResponse::new(),
        }
        .set_date(Utc::now())
    }

    pub fn new_with_version(version: HttpVersion) -> Self {
        Self::new().set_date(Utc::now()).set_version(version)
    }

    pub fn build(self) -> Result<HttpResponse> {
        if self.response.status.is_empty() {
            return Err(anyhow!("status must be set on response"));
        }

        trace!("{:?}", self.response);
        Ok(self.response)
    }

    pub fn set_version(mut self, version: HttpVersion) -> Self {
        self.response.version = version;
        self
    }

    pub fn set_status(self, status: HttpStatusCode) -> Self {
        self.set_raw_status(&status.to_string())
    }

    pub fn set_raw_status(mut self, status: &str) -> Self {
        self.response.status = status.to_owned();
        self
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.response
            .headers
            .insert(key.to_owned(), HttpHeader::new(key, value));
        self
    }

    pub fn set_cookie(mut self, cookie: HttpCookie) -> Self {
        self.response.cookies.insert(cookie.name.to_owned(), cookie);
        self
    }

    pub fn set_date(self, date: DateTime<Utc>) -> Self {
        let date = date.format("%a, %d %b %Y %H:%M:%S UTC").to_string();
        self.set_header("Date", &date)
    }

    pub fn set_content_type(self, content_type: &str) -> Self {
        self.set_header("Content-Type", content_type)
    }

    pub fn set_html_body(mut self, body: &str) -> Self {
        let body = format!("{}\r\n", body);
        let length = body.len().to_string();

        self.response.body = body.into_bytes();
        self.set_content_type("text/html")
            .set_header("Content-Length", &length)
    }

    pub fn set_json_body<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let body = serde_json::to_string(&body)?.to_string();
        let body = format!("{}\r\n", body);
        let length = body.len().to_string();

        self.response.body = body.into_bytes();
        Ok(self
            .set_content_type("application/json")
            .set_header("Content-Length", &length))
    }

    pub fn set_raw_body(mut self, body: Vec<u8>) -> Self {
        let length = body.len().to_string();

        self.response.body = body;
        self.set_content_type("application/octet-stream")
            .set_header("Content-Length", &length)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::cookie::SameSitePolicy;

    use super::*;

    #[test]
    fn test_cookie() {
        let expected = "HTTP/1.1 200 OK\r\n\
Content-Length: 20\r\n\
Content-Type: text/html\r\n\
Date: Tue, 29 Oct 2024 16:56:32 UTC\r\n\
Set-Cookie: User=jhondoe; SameSite=Lax; Secure\r\n\
Set-Cookie: foo=bar; HttpOnly; Path=/some/path\r\n\r\n<p>Hello World</p>\r\n"
            .as_bytes();

        let expires = DateTime::parse_from_rfc2822("Tue, 29 Oct 2024 16:56:32 +0000")
            .unwrap()
            .with_timezone(&Utc);

        let actual = HttpResponseBuilder::new()
            .set_status(HttpStatusCode::OK)
            .set_date(expires)
            .set_html_body("<p>Hello World</p>")
            .set_cookie(
                HttpCookie::new("foo", "bar")
                    .set_http_only(true)
                    .set_path(Some("/some/path")),
            )
            .set_cookie(
                HttpCookie::new("User", "jhondoe")
                    .set_secure(true)
                    .set_same_site(Some(SameSitePolicy::Lax)),
            )
            .build()
            .unwrap()
            .to_bytes()
            .unwrap();

        assert_eq!(expected, actual);
    }
}
