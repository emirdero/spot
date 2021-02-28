use crate::request;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

pub struct HttpParser {}

impl HttpParser {
    /// Parses an inncomming http request from a tcp stream, either returns the request object or an error string if the parse fails.
    pub fn parse(stream: &TcpStream) -> Result<request::Request, String> {
        let mut reader = BufReader::new(stream);
        // Read first line
        let mut http_request_line = String::new();
        let _result = reader.by_ref().read_line(&mut http_request_line);
        let http_request_line_split: Vec<&str> = http_request_line.split_whitespace().collect();

        // Input validation
        if http_request_line_split.len() != 3 {
            return Err(format!(
                "Request line not correct syntax: {}",
                http_request_line
            ));
        }

        // Process headers and body
        let mut http_request_headers = HashMap::new();
        for line_result in reader.by_ref().lines() {
            let line = match line_result {
                Ok(line_string) => line_string,
                Err(_error) => String::from("ERROR"),
            };
            if line == "" {
                break;
            }
            if line == "ERROR" {
                return Err(String::from("Failed to read line from TCP stream"));
            }
            let mut iter = line.split(": ");
            let key = match iter.next() {
                Some(result) => result,
                None => "Error: no key",
            };
            if key == "Error: no key" {
                return Err(String::from("Faulty request syntax, could not parse"));
            }
            let value = match iter.next() {
                Some(result) => result,
                None => "Error: no value",
            };
            if value == "Error: no value" {
                return Err(String::from("Faulty request syntax, could not parse"));
            }
            http_request_headers.insert(key.to_lowercase(), String::from(value));
        }
        let body;
        // If content lenght header is set we assume it has a body and try to read it
        if http_request_headers.contains_key("content-length") {
            let body_length: i32 = match http_request_headers["content-length"].parse() {
                Ok(result) => result,
                Err(_error) => -1,
            };
            if body_length < 0 {
                return Err(String::from("Invalid content-legth header"));
            }
            let mut body_bytes = vec![0; body_length as usize];
            let result = reader.by_ref().read_exact(&mut body_bytes);
            let fail = match result {
                Ok(_result) => false,
                Err(_error) => true,
            };
            if fail {
                return Err(String::from("Read request body failed"));
            } else {
                body = body_bytes;
            }
        } else {
            body = Vec::new();
        }

        // Get parameters from request
        let params_split: Vec<&str> = http_request_line_split[1].split("?").collect();
        let mut parameters = HashMap::new();
        // If the url has parameters set, get them
        if params_split.len() > 1 {
            let parameters_vec: Vec<&str> = params_split[1].split("&").collect();
            for parameter in parameters_vec {
                let parameter_split: Vec<&str> = parameter.split("=").collect();
                if parameter_split.len() > 1 {
                    parameters.insert(
                        String::from(parameter_split[0]),
                        String::from(parameter_split[1]),
                    );
                }
            }
        }
        let mut http_version = String::from("1.1");
        let version_split: Vec<&str> = http_request_line_split[2].split("/").collect();
        if version_split.len() > 1 {
            http_version = String::from(version_split[1]);
        }

        // Make method uppercase
        let mut method = String::from(http_request_line_split[0]);
        method.make_ascii_uppercase();
        Ok(request::Request::new(
            String::from(http_request_line_split[1]),
            parameters,
            body,
            http_version,
            method,
            http_request_headers,
        ))
    }
}
