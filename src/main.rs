use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};

use clap::Parser;

use crate::{
    http_request::{HttpMethod, HttpRequest},
    http_response::{Encoding, HttpResponse},
};

mod http_request;
mod http_response;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    directory: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let directory = cli.directory.clone();
                thread::spawn(move || {
                    process_request(stream, directory);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn process_request(mut stream: TcpStream, directory: Option<String>) {
    loop {
        let read_str = read_request(&mut stream);
        if let Some(read_str) = read_str {
            let request = HttpRequest::from_str(&read_str);

            dbg!(&request);

            let mut response = match &request.method {
                HttpMethod::Get => handle_get(&request, directory.clone()),
                HttpMethod::Post => handle_post(&request, directory.clone()),
            };

            let should_close = request.headers.get("Connection");
            if should_close == Some(&"close".to_string()) {
                response.add_header("Connection", "close");
            }

            dbg!(&response);

            stream.write_all(&response.to_bytes()).unwrap();

            if should_close == Some(&"close".to_string()) {
                break;
            }
        } else {
            break;
        }
    }
}

fn handle_get(request: &HttpRequest, directory: Option<String>) -> HttpResponse {
    match request.path.as_str() {
        "/" => HttpResponse::new(200, "OK"),
        "/user-agent" => {
            let user_agent = request.headers.get("User-Agent").unwrap();
            let mut response = HttpResponse::new(200, "OK");
            response.set_content(user_agent.to_string(), "text/plain");

            response
        }
        s if s.starts_with("/files/") => {
            let filename = s.trim_start_matches("/files/");
            let content = find_file(directory, filename);
            if let Some(content) = content {
                let mut response = HttpResponse::new(200, "OK");
                response.set_content(content, "application/octet-stream");
                response
            } else {
                HttpResponse::new(404, "Not Found")
            }
        }
        s if s.starts_with("/echo/") => {
            let echo_value = s.trim_start_matches("/echo/");
            let mut response = HttpResponse::new(200, "OK");
            response.set_content(echo_value.to_string(), "text/plain");
            if let Some(supported_encodings) = request.headers.get("Accept-Encoding") {
                let encoding: HashSet<&str> = supported_encodings.split(", ").collect();
                if encoding.contains("gzip") {
                    response.set_encoding(Encoding::Gzip);
                }
            }
            response
        }
        _ => HttpResponse::new(404, "Not Found"),
    }
}

fn handle_post(request: &HttpRequest, directory: Option<String>) -> HttpResponse {
    match request.path.as_str() {
        s if s.starts_with("/files/") => {
            let filename = s.trim_start_matches("/files/");
            write_file(directory, filename, &request.body);
            HttpResponse::new(201, "Created")
        }
        _ => HttpResponse::new(404, "Not Found"),
    }
}

fn find_file(directory: Option<String>, filename: &str) -> Option<String> {
    if let Some(directory) = directory {
        let mut path = PathBuf::from(directory);
        path.push(filename);
        if path.exists() {
            return Some(fs::read_to_string(path).unwrap());
        }
    }

    None
}

fn write_file(directory: Option<String>, filename: &str, content: &str) {
    if let Some(directory) = directory {
        let mut path = PathBuf::from(directory);
        path.push(filename);
        fs::write(path, content).unwrap();
    }
}

fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut request = String::new();
    loop {
        let mut buffer = [0u8; 1024];
        let n_bytes = stream.read(&mut buffer).unwrap();

        request.push_str(str::from_utf8(&buffer[..n_bytes]).unwrap());
        if n_bytes < 1024 {
            if !request.is_empty() {
                return Some(request);
            }
            return None;
        }
    }
}
