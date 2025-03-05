use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use serde::{Deserialize, Serialize};

use crate::http::{get_http_version, HttpVersion};

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub start_line: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub version: HttpVersion,
}

impl HttpRequest {
    pub fn from_tcp(stream: &TcpStream) -> anyhow::Result<HttpRequest> {
        let mut buf_reader = BufReader::new(stream);

        let mut start_line = String::new();
        let mut headers = HashMap::new();
        let mut body = String::new();

        buf_reader.read_line(&mut start_line)?;

        let version = get_http_version(&start_line)?;
        if version != HttpVersion::HTTP0_9 {
            let mut line = String::new();
            while buf_reader.read_line(&mut line)? > 0 {
                if line.trim().is_empty() {
                    break;
                }

                if let Some((key, value)) = line.trim_end().split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }

                line.clear();
            }

            if let Some(content_len) = headers.get("Content-Length") {
                let content_len: usize = content_len.parse()?;
                if content_len > 0 {
                    let mut buffer = vec![0; content_len];
                    buf_reader.read_exact(&mut buffer)?;
                    body = String::from_utf8(buffer)?;
                }
            }
        }

        let body = if body.len() > 0 { Some(body) } else { None };

        Ok(HttpRequest {
            start_line: start_line.trim().to_string(),
            headers,
            body,
            version,
        })
    }
}
