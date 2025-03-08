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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_version_0_9() {
        let expected = HttpVersion::HTTP0_9;
        let actual = HttpVersion::from_str("").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_to_str_version_0_9() {
        let expected = "";
        let actual = HttpVersion::HTTP0_9.to_string();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_str_version_1_0() {
        let expected = HttpVersion::HTTP1_0;
        let actual = HttpVersion::from_str("HTTP/1.0").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_to_str_version_1_0() {
        let expected = "HTTP/1.0";
        let actual = HttpVersion::HTTP1_0.to_string();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_str_version_1_1() {
        let expected = HttpVersion::HTTP1_1;
        let actual = HttpVersion::from_str("HTTP/1.1").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_to_str_version_1_1() {
        let expected = "HTTP/1.1";
        let actual = HttpVersion::HTTP1_1.to_string();

        assert_eq!(expected, actual);
    }
}
