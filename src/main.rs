use chrono::Utc;
use colored::Colorize;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use http_request::HttpRequest;

mod http;
mod http_request;

fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr).unwrap();
    println!("server started on {}", addr);
    println!("awaiting connections...");

    for stream in listener.incoming() {
        println!("{}", "new connection!".green());
        let stream = stream.unwrap();
        let result = handle_connection(stream);
        if let Err(result) = result {
            let error = format!("Error: {}", result);
            println!("{}", error.red());
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let request = HttpRequest::from_tcp(&stream)?;

    println!("{}", ">>> Request START <<<".red());
    println!(
        "{} {} {}",
        request.verb.to_string(),
        request.resource_path,
        request.version.to_string()
    );

    println!("{}", ">>> HEADERS <<<".red());
    for (key, value) in request.headers.iter() {
        println!("{}: {}", key, value);
    }

    if let Some(ref body) = request.body {
        println!("{}", ">>> BODY <<<".red());
        println!("{}", body);
    }

    println!("{}", ">>> Request END <<<".red());

    let request_json = serde_json::to_string(&request)?;
    let response = create_mirror_http11_response(&request_json);
    println!("{}", "dummy response sent!".blue());

    stream.write_all(response.as_bytes()).unwrap();
    Ok(())
}

fn create_mirror_http11_response(request_json: &str) -> String {
    let response = "HTTP/1.1 200 OK
Content-Type: application/json";

    let now = Utc::now();
    let date = now.format("Date: %a, %d %b %Y %H:%M:%S UTC").to_string();

    let response = format!("{response}\r\n{date}\r\n\r\n{request_json}");
    response.to_string()
}
