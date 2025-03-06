use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum HttpVerb {
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

impl FromStr for HttpVerb {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "GET" => HttpVerb::GET,
            "HEAD" => HttpVerb::HEAD,
            "POST" => HttpVerb::POST,
            "PUT" => HttpVerb::PUT,
            "DELETE" => HttpVerb::DELETE,
            "CONNECT" => HttpVerb::CONNECT,
            "OPTIONS" => HttpVerb::OPTIONS,
            "TRACE" => HttpVerb::TRACE,
            "PATCH" => HttpVerb::PATCH,
            value => return Err(anyhow!("unknown http verb: {}", value)),
        })
    }
}

impl ToString for HttpVerb {
    fn to_string(&self) -> String {
        match self {
            HttpVerb::GET => "GET",
            HttpVerb::HEAD => "HEAD",
            HttpVerb::POST => "POST",
            HttpVerb::PUT => "PUT",
            HttpVerb::DELETE => "DELETE",
            HttpVerb::CONNECT => "CONNECT",
            HttpVerb::OPTIONS => "OPTIONS",
            HttpVerb::TRACE => "TRACE",
            HttpVerb::PATCH => "PATCH",
        }
        .to_string()
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum HttpVersion {
    HTTP0_9,
    HTTP1_0,
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

pub fn parse_http_request_start_line(start_line: &str) -> Result<(HttpVerb, String, HttpVersion)> {
    let mut parts = start_line.split(" ").into_iter();

    let verb = parts
        .next()
        .context("start line should have HTTP verb")?
        .trim();

    let verb = HttpVerb::from_str(verb)?;

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
