use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::debug;
use serde::Serialize;

use super::{HttpCookie, HttpHeader, HttpResponse, HttpVersion};

pub struct HttpResponseBuilder {
    response: HttpResponse,
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
        debug!("{:#?}", self.response);
        if self.response.status.is_empty() {
            return Err(anyhow!("status must be set on response"));
        }
        Ok(self.response)
    }

    pub fn set_version(mut self, version: HttpVersion) -> Self {
        self.response.version = version;
        self
    }

    pub fn set_status(mut self, status: &str) -> Self {
        self.response.status = status.to_string();
        self
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.response
            .headers
            .insert(key.to_string(), HttpHeader::new(key, value));
        self
    }

    pub fn set_cookie(mut self, cookie: HttpCookie) -> Self {
        self.response
            .cookies
            .insert(cookie.name.to_string(), cookie);
        self
    }

    pub fn set_date(mut self, date: DateTime<Utc>) -> Self {
        let date = date.format("%a, %d %b %Y %H:%M:%S UTC").to_string();
        self = self.set_header("Date", &date);
        self
    }

    pub fn set_html_body(mut self, body: &str) -> Self {
        let body = format!("{}\r\n", body.to_string());
        let length = body.len().to_string();

        self.response.body = body;
        self = self.set_header("Content-Type", "text/html");
        self = self.set_header("Content-Length", &length);

        self
    }

    pub fn set_json_body<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let body = serde_json::to_string(&body)?.to_string();
        let body = format!("{}\r\n", body.to_string());
        let length = body.len().to_string();

        self.response.body = body;
        self = self.set_header("Content-Type", "application/json");
        self = self.set_header("Content-Length", &length);
        Ok(self)
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
Set-Cookie: foo=bar; HttpOnly; Path=/some/path\r\n\r\n<p>Hello World</p>\r\n";

        let expires = DateTime::parse_from_rfc2822("Tue, 29 Oct 2024 16:56:32 +0000")
            .unwrap()
            .with_timezone(&Utc);

        let actual = HttpResponseBuilder::new()
            .set_date(expires)
            .set_html_body("<p>Hello World</p>")
            .set_cookie(
                HttpCookie::new("foo", "bar")
                    .set_http_only(true)
                    .set_path(Some("/some/path".to_string())),
            )
            .set_cookie(
                HttpCookie::new("User", "jhondoe")
                    .set_secure(true)
                    .set_same_site(Some(SameSitePolicy::Lax)),
            )
            .build()
            .unwrap()
            .to_string()
            .unwrap();

        assert_eq!(expected, actual);
    }
}
