use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response = process_request(&mut stream);
                stream.write_all(&response).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn process_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = [0u8; 1024];
    let n_bytes = stream.read(&mut buffer).unwrap();

    let read_str = String::from_utf8(buffer[..n_bytes].to_vec()).unwrap();

    let parts: Vec<&str> = read_str.split("\r\n").collect();

    let mut headers = vec![];
    for line in &parts[1..] {
        if line.is_empty() {
            break;
        }
        headers.push(*line);
    }
    let mut header_map = HashMap::new();
    for header in headers {
        let parts: Vec<&str> = header.split(": ").collect();
        header_map.insert(parts[0], parts[1]);
    }

    let request_line: Vec<&str> = parts[0].split(" ").collect();
    dbg!(&parts, &header_map);
    match request_line[1] {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec(),
        "/user-agent" => {
            let user_agent = header_map.get("User-Agent").unwrap();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                *user_agent
            )
            .into_bytes()
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
