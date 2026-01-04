use std::collections::HashMap;

pub struct HttpResponse {
    status_code: u64,
    status: String,
    headers: HashMap<String, String>,
    content: String,
}

impl HttpResponse {
    pub fn new(status_code: u64, status: &str) -> Self {
        Self {
            status_code,
            status: status.to_string(),
            headers: HashMap::new(),
            content: String::from(""),
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn set_content(&mut self, content: String, content_type: &str) {
        self.headers
            .insert(String::from("Content-Type"), content_type.to_string());
        self.headers
            .insert(String::from("Content-Length"), content.len().to_string());
        self.content = content;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\n{}\r\n{}",
            self.status_code,
            self.status,
            self.format_headers(),
            self.content
        )
        .into_bytes()
    }

    fn format_headers(&self) -> String {
        let headers: Vec<String> = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        headers.join("\r\n")
    }
}
