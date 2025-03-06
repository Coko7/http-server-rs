use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::debug;
use serde::Serialize;

use crate::{http::HttpVersion, http_response::HttpResponse};

pub struct HttpResponseBuilder {
    response: HttpResponse,
}

impl HttpResponseBuilder {
    pub fn new() -> Result<Self> {
        HttpResponseBuilder {
            response: HttpResponse::new(),
        }
        .set_date(Utc::now())
    }

    pub fn new_with_version(version: HttpVersion) -> Result<Self> {
        Self::new()?.set_date(Utc::now())?.set_version(version)
    }

    pub fn build(self) -> Result<HttpResponse> {
        debug!("{:#?}", self.response);
        if self.response.status.is_empty() {
            return Err(anyhow!("status must be set on response"));
        }
        Ok(self.response)
    }

    pub fn set_version(mut self, version: HttpVersion) -> Result<Self> {
        self.response.version = version;
        Ok(self)
    }

    pub fn set_status(mut self, status: &str) -> Result<Self> {
        self.response.status = status.to_string();
        Ok(self)
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Result<Self> {
        self.response
            .headers
            .insert(key.to_string(), value.to_string());
        Ok(self)
    }

    pub fn set_cookie(mut self, key: &str, value: &str) -> Result<Self> {
        let cookie = format!("{}={}", key, value);
        self.response
            .headers
            .insert("Set-Cookie".to_string(), cookie);
        Ok(self)
    }

    pub fn set_date(mut self, date: DateTime<Utc>) -> Result<Self> {
        let date = date.format("%a, %d %b %Y %H:%M:%S UTC").to_string();
        self = self.set_header("Date", &date)?;
        Ok(self)
    }

    pub fn set_html_body(mut self, body: &str) -> Result<Self> {
        let body = format!("{}\r\n", body.to_string());
        let length = body.len().to_string();

        self.response.body = body;
        self = self.set_header("Content-Type", "text/html")?;
        self = self.set_header("Content-Length", &length)?;

        Ok(self)
    }

    pub fn set_json_body<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let body = serde_json::to_string(&body)?.to_string();
        let body = format!("{}\r\n", body.to_string());
        let length = body.len().to_string();

        self.response.body = body;
        self = self.set_header("Content-Type", "application/json")?;
        self = self.set_header("Content-Length", &length)?;
        Ok(self)
    }
}
