use std::collections::HashMap;
pub struct Request {
    pub url: String,
    pub params: HashMap<String, String>,
    pub body: String,
    pub http_version: String,
    pub method: String,
    pub headers: HashMap<String, String>,
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

    pub fn contains_params(&self, keys: Vec<&str>) -> bool {
        for key in keys {
            if !self.params.contains_key(key) {
                return false;
            }
        }
        return true;
    }
}
