use std::collections::HashMap;

pub struct Response {
    body: String,
    headers: HashMap<String, String>,
}

impl Response {
    pub fn new(body: String, headers: HashMap<String, String>) -> Request {
        return Response {
            body: body,
            headers: headers,
        };
    }
}