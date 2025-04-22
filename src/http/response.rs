use anyhow::{bail, Result};
use log::trace;
use std::collections::BTreeMap;

use super::{HttpCookie, HttpHeader, HttpVersion};

#[derive(Debug)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: String,
    pub headers: BTreeMap<String, HttpHeader>,
    pub cookies: BTreeMap<String, HttpCookie>,
    pub body: Vec<u8>,
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
            body: Vec::new(),
        }
    }

    pub fn start_line(&self) -> String {
        format!("{} {}", self.version, self.status)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        if self.status.is_empty() {
            bail!("status must be set on response");
        }

        let mut head = format!("{}\r\n", self.start_line());
        trace!("{:?}", head);

        for (_, header) in self.headers.iter() {
            let header = format!("{}: {}\r\n", header.name, header.value);
            head.push_str(&header);
        }

        for (_, cookie) in self.cookies.iter() {
            let cookie = cookie.to_str()?;
            let header = format!("Set-Cookie: {}\r\n", cookie);
            head.push_str(&header);
        }

        head.push_str("\r\n");

        let response_head = head.as_bytes();
        let body = &self.body;
        let response: Vec<u8> = [response_head, body].concat();

        Ok(response)
    }
}
