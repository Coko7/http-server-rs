use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use log::trace;

use super::{HttpCookie, HttpHeader, HttpVersion};

#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: String,
    pub headers: BTreeMap<String, HttpHeader>,
    pub cookies: BTreeMap<String, HttpCookie>,
    pub body: String,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status: "200 OK".to_owned(),
            headers: BTreeMap::new(),
            cookies: BTreeMap::new(),
            body: String::new(),
        }
    }

    pub fn start_line(&self) -> String {
        format!("{} {}", self.version, self.status)
    }

    pub fn to_string(&self) -> Result<String> {
        if self.status.is_empty() {
            return Err(anyhow!("status must be set on response"));
        }
        let mut response = format!("{}\r\n", self.start_line());
        trace!("{:?}", response);

        for (_, header) in self.headers.iter() {
            let header = format!("{}: {}\r\n", header.name, header.value);
            response.push_str(&header);
        }

        for (_, cookie) in self.cookies.iter() {
            let cookie = cookie.to_str()?;
            let header = format!("Set-Cookie: {}\r\n", cookie);
            response.push_str(&header);
        }

        response.push_str("\r\n");
        response.push_str(&self.body);

        Ok(response)
    }
}
