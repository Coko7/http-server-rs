use anyhow::Result;
use log::trace;
use std::{
    io::{BufRead, BufReader, Read},
    net::{IpAddr, TcpStream},
};

use super::HttpHeader;

pub struct HttpRequestRaw {
    pub request_line: String,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
    pub peer_ip: IpAddr,
    pub local_ip: IpAddr,
}

impl HttpRequestRaw {
    pub fn from_tcp(stream: &TcpStream) -> Result<HttpRequestRaw> {
        trace!("trying to convert TCP message into HTTP request");
        let mut buf_reader = BufReader::new(stream);

        let peer_ip = stream.peer_addr()?.ip();
        let local_ip = stream.local_addr()?.ip();

        let mut request_line = String::new();
        let mut headers = Vec::new();
        let mut body = Vec::new();

        trace!("read request line");
        buf_reader.read_line(&mut request_line)?;

        let mut line = String::new();
        trace!("proceed to read read headers");
        while buf_reader.read_line(&mut line)? > 0 {
            if line.trim().is_empty() {
                break;
            }

            if let Some((key, value)) = line.trim_end().split_once(':') {
                let header = HttpHeader {
                    name: key.trim().to_owned(),
                    value: value.trim().to_owned(),
                };
                headers.push(header);
            }

            line.clear();
        }

        if let Some(content_len) = headers
            .iter()
            .find(|header| header.name == "Content-Length")
        {
            trace!("found Content-Length header, using value to read body");
            let content_len: usize = content_len.value.parse()?;
            if content_len > 0 {
                trace!("read body ({} bytes)", content_len);
                body = vec![0; content_len];
                buf_reader.read_exact(&mut body)?;
            }
        }

        trace!("finish processing TCP stream");
        Ok(HttpRequestRaw {
            request_line,
            headers,
            body,
            peer_ip,
            local_ip,
        })
    }
}
