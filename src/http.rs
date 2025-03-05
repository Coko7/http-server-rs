use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum HttpVersion {
    HTTP0_9,
    HTTP1_0,
    HTTP1_1,
}

pub fn get_http_version(start_line: &str) -> anyhow::Result<HttpVersion> {
    let mut parts = start_line.split(" ").into_iter();

    let _verb = parts.next().context("start line should have HTTP verb")?;
    let _resource_path = parts
        .next()
        .context("start line should have resource path")?;

    if let Some(version) = parts.next() {
        return match version.trim() {
            "HTTP/1.0" => Ok(HttpVersion::HTTP1_0),
            "HTTP/1.1" => Ok(HttpVersion::HTTP1_1),
            _ => Err(anyhow!("unsupported HTTP version: {}", version)),
        };
    } else {
        return Ok(HttpVersion::HTTP0_9);
    }
}
