use anyhow::{Context, Result};
use log::debug;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use serde::{Deserialize, Serialize};

use crate::http::{parse_http_request_start_line, HttpVerb, HttpVersion};

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub raw_start_line: String,

    pub verb: HttpVerb,
    pub resource_path: String,
    pub version: HttpVersion,

    pub url: String,
    pub query_params: HashMap<String, String>,

    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn from_tcp(stream: &TcpStream) -> Result<HttpRequest> {
        let mut buf_reader = BufReader::new(stream);

        let mut start_line = String::new();
        let mut headers = HashMap::new();
        let mut body = String::new();

        buf_reader.read_line(&mut start_line)?;

        debug!("start line: {}", start_line.trim());
        let (verb, resource_path, version) = parse_http_request_start_line(&start_line)?;

        let query_params = if resource_path.contains("?") {
            let (_, query_line) = resource_path.split_once('?').unwrap();
            parse_query_line(&query_line)?
        } else {
            HashMap::new()
        };

        let url = resource_path
            .split('?')
            .next()
            .unwrap_or(&resource_path)
            .to_string();

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
            raw_start_line: start_line.trim().to_string(),
            headers,
            body,
            version,
            verb,
            resource_path,
            query_params,
            url,
        })
    }
}

fn parse_query_line(resource_path: &str) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    let query_params = resource_path.split("&");

    for param in query_params {
        let (key, value) = param.split_once('=').context("= should be in query")?;
        result.insert(key.to_string(), value.to_string());
    }

    Ok(result)
}
