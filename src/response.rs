use std::collections::HashMap;

pub struct Response {
    /// Status code for the response
    pub status: u16,
    /// The response body. If there is no body this is empty
    pub body: Vec<u8>,
    /// Contains all the desired response headers. No headers are added automatically except for content-length and content-type when adding a body
    pub headers: HashMap<String, String>,
}

impl Response {
    /// Creates a new http response object
    pub fn new(status: u16, body: Vec<u8>, headers: HashMap<String, String>) -> Response {
        return Response {
            status: status,
            body: body,
            headers: headers,
        };
    }

    /// Updates the response status
    pub fn status(&mut self, new_status: u16) {
        self.status = new_status;
    }

    /// Adds a new header to the response
    pub fn header(&mut self, name_ref: impl AsRef<str>, value_ref: impl AsRef<str>) {
        let name = name_ref.as_ref().to_string();
        let value = value_ref.as_ref().to_string();
        if self.headers.contains_key(&name) {
            self.headers.remove(&name);
        }
        self.headers.insert(name, value);
    }

    /// Adds a body from a String to the request, overwrites previous body, then adds content length and content type "text/plain".
    pub fn body(&mut self, data_ref: impl AsRef<str>) {
        let data = data_ref.as_ref().to_string();
        self.body = data.as_bytes().to_vec();
        // Sets the content length header so that the client knows how far to read
        self.header(String::from("content-length"), self.body.len().to_string());
        self.header(String::from("content-type"), String::from("text/plain"));
    }

    /// Adds a body from a byte array, then sets the content-length header to be equal to the size of the array
    pub fn body_bytes(&mut self, data: Vec<u8>) {
        self.body = data;
        self.header(String::from("content-length"), self.body.len().to_string());
    }

    /// Converts the reponse to an array of bytes in order to write it over the TCP stream. This always writes in the HTTP/1.1 format
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
