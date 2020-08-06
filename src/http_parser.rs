use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
mod request;

pub struct HttpParser {}

impl HttpParser {
    pub fn new() -> HttpParser {
        return HttpParser;
    }

    pub fn parse(stream: TcpStream) -> Result<request::Request, String> {
        let mut reader = BufReader::new(stream);
        // Read first line
        let _result = reader.by_ref().read_line(&mut http_request_definition);
        let http_request_definition_split: Vec<&str> = http_request_definition.split_whitespace().collect();

        // Process headers and body
        let mut http_request_headers = HashMap::new(String, String);
        let mut body_bytes: Vec<u8> = vec![];
        
        for line_result in reader.by_ref().lines() {
            let line = match line_result {
                Ok(line_string) => line_string,
                Err(error) => return Err(Failed to read line from TCP stream)
            }
            let mut iter = line.split(": ");
            let key = match iter.next() {
                Ok(result) => result,
                Err(error) => return Err("Faulty request syntax, could not parse")
            }
            let value = match iter.next() {
                Ok(result) => result,
                Err(error) => return Err("Faulty request syntax, could not parse")
            }
            http_request_headers.insert(key, value);
        }
        if has_body {
            let body_length = match .parse(){

            }
            body_bytes = vec![0; body_length];
            let result = reader.by_ref().read_exact(&mut body);
            match result {
                Ok() => {}
                Err(error) => {println!("Read request body failed: ", error)}
            }
        }
    }
}
