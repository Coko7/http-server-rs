use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::TcpStream, str::FromStr};

use crate::http::request_raw::HttpRequestRaw;

use super::{HttpCookie, HttpHeader, HttpMethod, HttpVersion};

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub resource_path: String,
    pub version: HttpVersion,

    pub url: String,
    pub query: HashMap<String, String>,

    pub headers: HashMap<String, HttpHeader>,
    pub cookies: HashMap<String, HttpCookie>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn from_raw_request(raw_request: HttpRequestRaw) -> Result<HttpRequest> {
        let (verb, resource_path, version) = Self::parse_request_line(&raw_request.request_line)?;

        let query_params = if resource_path.contains("?") {
            let (_, query_line) = resource_path
                .split_once('?')
                .context("resource path should contain query sep `?`")?;
            Self::parse_query_line(query_line)?
        } else {
            HashMap::new()
        };

        let url = resource_path
            .split('?')
            .next()
            .unwrap_or(&resource_path)
            .to_owned();

        let cookies: HashMap<String, HttpCookie> = raw_request
            .headers
            .iter()
            .filter(|header| header.name == "Cookie")
            .map(|header| header.value.to_owned())
            .map(|cookie_def| HttpCookie::from_cookie_line(&cookie_def).unwrap())
            .map(|cookie| (cookie.name.to_owned(), cookie))
            .collect();

        let headers: HashMap<String, HttpHeader> = raw_request
            .headers
            .into_iter()
            .filter(|header| header.name != "Cookie")
            .map(|header| (header.name.to_owned(), header))
            .collect();

        Ok(HttpRequest {
            headers,
            cookies,
            body: raw_request.body,
            version,
            method: verb,
            resource_path,
            query: query_params,
            url,
        })
    }

    pub fn from_tcp(stream: &TcpStream) -> Result<HttpRequest> {
        let raw_request = HttpRequestRaw::from_tcp(stream)?;
        Self::from_raw_request(raw_request)
    }

    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn str_body(&self) -> Result<String> {
        Ok(String::from_utf8(self.body.clone())?)
    }

    pub fn parse_request_line(start_line: &str) -> Result<(HttpMethod, String, HttpVersion)> {
        let mut parts = start_line.split(" ");

        let verb = parts
            .next()
            .context("start line should have HTTP verb")?
            .trim();

        let verb = HttpMethod::from_str(verb)?;

        let resource_path = parts
            .next()
            .context("start line should have resource path")?
            .trim()
            .to_owned();

        let version = if let Some(version) = parts.next() {
            HttpVersion::from_str(version.trim())?
        } else {
            return Err(anyhow!(
                "sorry... HTTP/0.9 is temporarily not supported for convenience"
            ));
        };

        Ok((verb, resource_path, version))
    }

    fn parse_query_line(resource_path: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        let query_params = resource_path.split("&");

        for param in query_params {
            let (key, value) = param.split_once('=').context("= should be in query")?;
            result.insert(key.to_owned(), value.to_owned());
        }

        Ok(result)
    }
}
