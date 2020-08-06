use std::collections::HashMap;

pub struct Request {
    url: String,
    params: HashMap<String, String>,
    body: String,
    http_version: String,
    method: String,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(
        url: String,
        params: HashMap<String, String>,
        body: String,
        http_version: String,
        method: String,
        headers: HashMap<String, String>,
    ) -> Request {
        return Request {
            url: url,
            params: params,
            body: body,
            http_version: http_version,
            method: method,
            headers: headers,
        };
    }
}
