use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::debug;

use crate::http::HttpVersion;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn start_line(&self) -> String {
        format!("{} {}", self.version.to_string(), self.status)
    }

    pub fn to_string(&self) -> Result<String> {
        if self.status.is_empty() {
            return Err(anyhow!("status must be set on response"));
        }
        let mut response = format!("{}\r\n", self.start_line());
        debug!("{:?}", response);

        for (key, value) in self.headers.iter() {
            let header = format!("{}: {}\r\n", key, value);
            response.push_str(&header);
        }

        response.push_str("\r\n\r\n");
        response.push_str(&self.body);

        Ok(response)
    }
}
