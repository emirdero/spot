use std::collections::HashMap;
pub struct Request {
    /// The url of the requested resource
    pub url: String,
    /// Contains the paramteres specified in the url
    ///
    /// for example /user?name=cory&age=21 would yield name and age as keys with cory and 21 as values respectively
    pub params: HashMap<String, String>,
    /// The body of the request if the request has specified a content-length header, otherwise the string is a fresh Vec::new()
    pub body: Vec<u8>,
    /// The http version. Note: Spot only supporst 1.1 at the moment
    pub http_version: String,
    /// The request method (GET, POST, PUT etc). Method should always be fully capitalized.
    pub method: String,
    /// Contains all the request headers. These are always all lower-case in Spot
    ///
    /// content-length: 120 would for example yield content-length as a key with value "120"
    pub headers: HashMap<String, String>,
}

impl Request {
    /// Create a new http request object.
    pub fn new(
        url: String,
        params: HashMap<String, String>,
        body: Vec<u8>,
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
    /// Check if the http request contains the specified list of parameters.
    pub fn contains_params(&self, keys: Vec<&str>) -> bool {
        for key in keys {
            if !self.params.contains_key(key) {
                return false;
            }
        }
        return true;
    }
}
