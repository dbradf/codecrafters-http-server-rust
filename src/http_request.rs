use std::collections::HashMap;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

enum ParseState {
    RequestLine,
    Headers,
    Body,
}

impl HttpRequest {
    pub fn from_str(request: &str) -> Self {
        let mut method = HttpMethod::Get;
        let mut state = ParseState::RequestLine;
        let mut headers = HashMap::new();
        let mut path = None;
        let mut body = String::new();

        for line in request.split("\r\n") {
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
                    body.push_str(line);
                }
            }
        }

        HttpRequest {
            method,
            path: path.unwrap(),
            headers,
            body,
        }
    }
}
