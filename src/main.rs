use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};

use clap::Parser;

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
            Ok(mut stream) => {
                println!("accepted new connection");
                let directory = cli.directory.clone();
                thread::spawn(move || {
                    let response = process_request(&mut stream, directory);
                    stream.write_all(&response).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn process_request(stream: &mut TcpStream, directory: Option<String>) -> Vec<u8> {
    let read_str = read_request(stream);
    let request = parse_request(&read_str);

    dbg!(&request);

    match request.path.as_str() {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec(),
        "/user-agent" => {
            let user_agent = request.headers.get("User-Agent").unwrap();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                *user_agent
            )
            .into_bytes()
        }
        s if s.starts_with("/files/") => {
            let filename = s.trim_start_matches("/files/");
            let content = find_file(directory, filename);
            if let Some(content) = content {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                    content.len(), content
                ).into_bytes()
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec()
            }
        }
        s if s.starts_with("/echo/") => {
            let echo_value = s.trim_start_matches("/echo/");
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                echo_value.len(),
                echo_value
            )
            .as_bytes()
            .to_vec()
        }
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec(),
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

fn read_request(stream: &mut TcpStream) -> String {
    let mut request = String::new();
    loop {
        let mut buffer = [0u8; 1024];
        let n_bytes = stream.read(&mut buffer).unwrap();

        request.push_str(str::from_utf8(&buffer[..n_bytes]).unwrap());
        if n_bytes < 1024 {
            return request;
        }
    }
}

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug)]
struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>,
}

enum ParseState {
    RequestLine,
    Headers,
    Body,
}

fn parse_request(request: &str) -> HttpRequest {
    let mut method = HttpMethod::Get;
    let mut state = ParseState::RequestLine;
    let mut headers = HashMap::new();
    let mut path = None;
    let parts: Vec<&str> = request.split("\r\n").collect();

    for line in parts {
        match state {
            ParseState::RequestLine => {
                let request_line: Vec<&str> = line.split(" ").collect();
                if request_line[0] == "POST" {
                    method = HttpMethod::Post;
                }

                path = Some(request_line[1].to_string());

                state = ParseState::Headers;
            }
            ParseState::Headers => {
                if line.is_empty() {
                    state = ParseState::Body;
                    continue;
                }

                let parts: Vec<&str> = line.split(": ").collect();
                headers.insert(parts[0].to_string(), parts[1].to_string());
            }
            ParseState::Body => {
                continue;
            }
        }
    }

    HttpRequest {
        method,
        path: path.unwrap(),
        headers,
    }
}
