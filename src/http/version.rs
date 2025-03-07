use anyhow::anyhow;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
