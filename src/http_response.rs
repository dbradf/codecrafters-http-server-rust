use std::collections::HashMap;
use std::io::prelude::*;

use flate2::{Compression, write::GzEncoder};

#[derive(Debug)]
pub enum Encoding {
    Gzip,
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Gzip => "gzip".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    status_code: u64,
    status: String,
    headers: HashMap<String, String>,
    content: String,
    encoding: Option<Encoding>,
}

impl HttpResponse {
    pub fn new(status_code: u64, status: &str) -> Self {
        Self {
            status_code,
            status: status.to_string(),
            headers: HashMap::new(),
            content: String::from(""),
            encoding: None,
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

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.add_header("Content-Encoding", &encoding.to_string());
        self.encoding = Some(encoding);
    }

    pub fn format_content(&self) -> Vec<u8> {
        if let Some(encoding) = &self.encoding {
            match encoding {
                Encoding::Gzip => {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(self.content.as_bytes()).unwrap();
                    encoder.finish().unwrap()
                }
            }
        } else {
            self.content.clone().into_bytes()
        }
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        let content = self.format_content();
        self.add_header("Content-Length", &content.len().to_string());
        let mut bytes = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n",
            self.status_code,
            self.status,
            self.format_headers(),
        )
        .into_bytes();
        bytes.extend(self.format_content());
        bytes
    }

    fn format_headers(&self) -> String {
        let headers: Vec<String> = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect();

        headers.join("")
    }
}
