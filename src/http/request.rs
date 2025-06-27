use anyhow::{bail, Context, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, TcpStream},
    str::FromStr,
};

use super::{HttpCookie, HttpHeader, HttpMethod, HttpRequestRaw, HttpVersion, MultipartBody};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub resource_path: String,
    pub version: HttpVersion,

    pub url: String,
    pub query: HashMap<String, String>,

    pub headers: HashMap<String, HttpHeader>,
    pub cookies: HashMap<String, HttpCookie>,
    pub body: Vec<u8>,

    pub peer_ip: IpAddr,
    pub local_ip: IpAddr,
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
            .flat_map(|cookie_def| HttpCookie::from_req_header_cookie_line(&cookie_def).unwrap())
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
            peer_ip: raw_request.peer_ip,
            local_ip: raw_request.local_ip,
        })
    }

    pub fn from_tcp(stream: &TcpStream) -> Result<HttpRequest> {
        let raw_request = HttpRequestRaw::from_tcp(stream)?;
        Self::from_raw_request(raw_request)
    }

    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn get_str_body(&self) -> Result<String> {
        Ok(String::from_utf8(self.body.clone())?)
    }

    pub fn get_multipart_body(&self) -> Result<MultipartBody> {
        let content_type = self
            .headers
            .get("Content-Type")
            .context("cannot process multipart body because Content-Type header is missing")?;

        let multipart_boundary = content_type
            .value
            .strip_prefix("multipart/form-data; boundary=")
            .context("boundary is required with multipart body")?;

        trace!("header boundary: {multipart_boundary}");

        MultipartBody::from_bytes(multipart_boundary, &self.body)
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
            bail!("sorry... HTTP/0.9 is temporarily not supported for convenience");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let expected = (HttpMethod::GET, "/home".to_owned(), HttpVersion::HTTP1_1);
        let actual = HttpRequest::parse_request_line("GET /home HTTP/1.1").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_query_line() {
        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert("query".to_owned(), "This+is+a+query".to_owned());
        expected.insert("mode".to_owned(), "foo".to_owned());
        expected.insert("Format".to_owned(), "json".to_owned());

        let query_line = "query=This+is+a+query&mode=foo&Format=json";
        let actual = HttpRequest::parse_query_line(&query_line).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_simple_get() {
        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/api/weather".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/api/weather".to_owned(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "GET /api/weather HTTP/1.1".to_owned(),
            headers: vec![],
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_get_with_query() {
        let mut query_params = HashMap::new();
        query_params.insert("country".to_owned(), "France".to_owned());
        query_params.insert("city".to_owned(), "Paris".to_owned());

        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/api/weather?country=France&city=Paris".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/api/weather".to_owned(),
            query: query_params,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "GET /api/weather?country=France&city=Paris HTTP/1.1".to_owned(),
            headers: vec![],
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_get_with_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_owned(),
            HttpHeader::new("Authorization", "Bearer JWT"),
        );
        headers.insert(
            "X-CSRF-Token".to_owned(),
            HttpHeader::new("X-CSRF-Token", "HelloWorld"),
        );

        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/api/weather".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/api/weather".to_owned(),
            query: HashMap::new(),
            headers: headers.clone(),
            cookies: HashMap::new(),
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let headers_vec: Vec<HttpHeader> = headers.values().cloned().collect();
        let raw_request = HttpRequestRaw {
            request_line: "GET /api/weather HTTP/1.1".to_owned(),
            headers: headers_vec,
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_post_body() {
        let body_bytes = "username:john,password:doe".as_bytes();

        let expected = HttpRequest {
            method: HttpMethod::POST,
            resource_path: "/users".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/users".to_owned(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: body_bytes.to_vec(),
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "POST /users HTTP/1.1".to_owned(),
            headers: vec![],
            body: body_bytes.to_vec(),
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_one_cookie() {
        let mut cookies: HashMap<String, HttpCookie> = HashMap::new();
        cookies.insert(String::from("foo"), HttpCookie::new("foo", "foov"));

        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/users".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/users".to_owned(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: cookies,
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "GET /users HTTP/1.1".to_owned(),
            headers: vec![HttpHeader::new("Cookie", "foo=foov")],
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_multi_cookies() {
        let mut cookies: HashMap<String, HttpCookie> = HashMap::new();
        cookies.insert(String::from("foo"), HttpCookie::new("foo", "foov"));
        cookies.insert(String::from("bar"), HttpCookie::new("bar", "barv"));

        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/users".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/users".to_owned(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: cookies,
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "GET /users HTTP/1.1".to_owned(),
            headers: vec![
                HttpHeader::new("Cookie", "foo=foov"),
                HttpHeader::new("Cookie", "bar=barv"),
            ],
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_raw_request_multi_cookies_same_header() {
        let mut cookies: HashMap<String, HttpCookie> = HashMap::new();
        cookies.insert(String::from("foo"), HttpCookie::new("foo", "foov"));
        cookies.insert(String::from("bar"), HttpCookie::new("bar", "barv"));

        let expected = HttpRequest {
            method: HttpMethod::GET,
            resource_path: "/users".to_owned(),
            version: HttpVersion::HTTP1_1,
            url: "/users".to_owned(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: cookies,
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let raw_request = HttpRequestRaw {
            request_line: "GET /users HTTP/1.1".to_owned(),
            headers: vec![HttpHeader::new("Cookie", "foo=foov; bar=barv")],
            body: vec![],
            peer_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            local_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        };

        let actual = HttpRequest::from_raw_request(raw_request).unwrap();
        assert_eq!(expected, actual);
    }
}
