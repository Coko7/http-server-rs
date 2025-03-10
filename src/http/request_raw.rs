use anyhow::Result;
use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use super::HttpHeader;

pub struct HttpRequestRaw {
    pub request_line: String,
    pub headers: Vec<HttpHeader>,
    pub body: Option<String>,
}

impl HttpRequestRaw {
    pub fn from_tcp(stream: &TcpStream) -> Result<HttpRequestRaw> {
        let mut buf_reader = BufReader::new(stream);

        let mut request_line = String::new();
        let mut headers = Vec::new();
        let mut body = String::new();

        buf_reader.read_line(&mut request_line)?;

        let mut line = String::new();
        while buf_reader.read_line(&mut line)? > 0 {
            if line.trim().is_empty() {
                break;
            }

            if let Some((key, value)) = line.trim_end().split_once(':') {
                let header = HttpHeader {
                    name: key.trim().to_string(),
                    value: value.trim().to_string(),
                };
                headers.push(header);
            }

            line.clear();
        }

        if let Some(content_len) = headers
            .iter()
            .find(|header| header.name == "Content-Length")
        {
            let content_len: usize = content_len.value.parse()?;
            if content_len > 0 {
                let mut buffer = vec![0; content_len];
                buf_reader.read_exact(&mut buffer)?;
                body = String::from_utf8(buffer)?;
            }
        }

        let body = if body.len() > 0 { Some(body) } else { None };

        Ok(HttpRequestRaw {
            request_line,
            headers,
            body,
        })
    }
}
