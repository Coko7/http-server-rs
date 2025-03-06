use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "GET" => HttpMethod::GET,
            "HEAD" => HttpMethod::HEAD,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "CONNECT" => HttpMethod::CONNECT,
            "OPTIONS" => HttpMethod::OPTIONS,
            "TRACE" => HttpMethod::TRACE,
            "PATCH" => HttpMethod::PATCH,
            value => return Err(anyhow!("unknown http verb: {}", value)),
        })
    }
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::PATCH => "PATCH",
        }
        .to_string()
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum HttpVersion {
    #[serde(rename = "HTTP/0.9")]
    HTTP0_9,
    #[serde(rename = "HTTP/1.0")]
    HTTP1_0,
    #[serde(rename = "HTTP/1.1")]
    HTTP1_1,
}

impl FromStr for HttpVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "" => HttpVersion::HTTP0_9,
            "HTTP/1.0" => HttpVersion::HTTP1_0,
            "HTTP/1.1" => HttpVersion::HTTP1_1,
            value => return Err(anyhow!("unsupported HTTP version: {}", value)),
        })
    }
}

impl ToString for HttpVersion {
    fn to_string(&self) -> String {
        match self {
            HttpVersion::HTTP0_9 => "",
            HttpVersion::HTTP1_0 => "HTTP/1.0",
            HttpVersion::HTTP1_1 => "HTTP/1.1",
        }
        .to_string()
    }
}

pub fn parse_http_request_start_line(
    start_line: &str,
) -> Result<(HttpMethod, String, HttpVersion)> {
    let mut parts = start_line.split(" ").into_iter();

    let verb = parts
        .next()
        .context("start line should have HTTP verb")?
        .trim();

    let verb = HttpMethod::from_str(verb)?;

    let resource_path = parts
        .next()
        .context("start line should have resource path")?
        .trim()
        .to_string();

    let version = if let Some(version) = parts.next() {
        HttpVersion::from_str(version.trim())?
    } else {
        HttpVersion::HTTP0_9
    };

    Ok((verb, resource_path, version))
}
