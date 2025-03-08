use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use log::debug;

use super::{HttpCookie, HttpVersion};

#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub cookies: HashSet<HttpCookie>,
    pub body: String,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status: "200 OK".to_string(),
            headers: HashMap::new(),
            cookies: HashSet::new(),
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
