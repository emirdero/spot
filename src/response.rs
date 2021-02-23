use std::collections::HashMap;

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn new(status: u16, body: Vec<u8>, headers: HashMap<String, String>) -> Response {
        return Response {
            status: status,
            body: body,
            headers: headers,
        };
    }

    pub fn status(&mut self, new_status: u16) {
        self.status = new_status;
    }

    pub fn header(&mut self, name_ref: impl AsRef<str>, value_ref: impl AsRef<str>) {
        let name = name_ref.as_ref().to_string();
        let value = value_ref.as_ref().to_string();
        if self.headers.contains_key(&name) {
            self.headers.remove(&name);
        }
        self.headers.insert(name, value);
    }

    pub fn body(&mut self, data_ref: impl AsRef<str>) {
        let data = data_ref.as_ref().to_string();
        self.body = data.as_bytes().to_vec();
        // Sets the content length header so that the client knows how far to read
        self.header(String::from("content-length"), self.body.len().to_string());
    }

    pub fn body_bytes(&mut self, data: Vec<u8>) {
        self.body = data;
        self.header(String::from("content-length"), self.body.len().to_string());
    }

    pub fn to_http(self) -> Vec<u8> {
        let mut http: Vec<u8> = Vec::new();
        http.extend_from_slice(format!("HTTP/1.1 {} \n", self.status).as_bytes());
        for (key, value) in self.headers {
            http.extend_from_slice(format!("{}: {}\n", key, value).as_bytes());
        }
        // append newline
        http.push(10);
        http.extend_from_slice(&self.body);
        return http;
    }
}
