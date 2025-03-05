use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

pub struct HttpResponse {
    start_line: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new() -> Self {
        let response = HttpResponse {
            start_line: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
        .set_start_line("HTTP/1.1 200 OK")
        .unwrap()
        .set_date(Utc::now())
        .unwrap();

        response
    }

    pub fn set_start_line(mut self, start_line: &str) -> Result<Self> {
        self.start_line = start_line.to_string();
        Ok(self)
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Result<Self> {
        self.headers.insert(key.to_string(), value.to_string());
        Ok(self)
    }

    pub fn set_cookie(mut self, key: &str, value: &str) -> Result<Self> {
        let cookie = format!("{}={}", key, value);
        self.headers.insert("Set-Cookie".to_string(), cookie);
        Ok(self)
    }

    pub fn set_date(mut self, date: DateTime<Utc>) -> Result<Self> {
        let date = date.format("%a, %d %b %Y %H:%M:%S UTC").to_string();
        self = self.set_header("Date", &date)?;
        Ok(self)
    }

    pub fn set_html_body(mut self, body: &str) -> Result<Self> {
        self = self.set_header("Content-Type", "text/html")?;
        self.body = body.to_string();
        Ok(self)
    }

    pub fn set_json_body<T: Serialize>(mut self, body: &T) -> Result<Self> {
        self = self.set_header("Content-Type", "application/json")?;
        self.body = serde_json::to_string(&body)?.to_string();
        Ok(self)
    }
}

impl ToString for HttpResponse {
    fn to_string(&self) -> String {
        let mut response = format!("{}\r\n", self.start_line.to_string());

        for (key, value) in self.headers.iter() {
            let header = format!("{}: {}\r\n", key, value);
            response.push_str(&header);
        }

        response.push_str("\r\n\r\n");
        response.push_str(&self.body);

        response
    }
}
