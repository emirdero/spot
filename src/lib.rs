extern crate threadpool;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use threadpool::ThreadPool;
mod http_parser;
pub mod request;
pub mod response;
use http_parser::HttpParser;
use request::Request;
use response::Response;

pub struct Spot {
    pool: ThreadPool,
    routes: HashMap<String, fn(Request, Response) -> Response>,
}

impl Spot {
    pub fn new(amount_of_threads: usize) -> Spot {
        return Spot {
            pool: ThreadPool::new(amount_of_threads),
            routes: HashMap::new(),
        };
    }

    pub fn route(&mut self, path: &str, function: fn(request::Request, Response) -> Response) {
        if self.routes.contains_key(path) {
            println!("Warning: Route defined twice, using latest definition");
            self.routes.remove(path);
        }
        self.routes.insert(path.to_owned(), function);
    }

    pub fn bind(&mut self, ip: &str) -> String {
        let listener = match TcpListener::bind(ip) {
            Ok(result) => result,
            Err(error) => {
                return String::from(format!("Failed to bind to ip: {}", error));
            }
        };
        println!("Spot server listening on: http://{}", ip);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let routes_clone = self.routes.clone();
            self.pool.execute(|| {
                handle_request(stream, routes_clone);
            });
        }
        return String::from("Shutting down.");
    }
}

fn handle_request(stream: TcpStream, routes: HashMap<String, fn(Request, Response) -> Response>) {
    let mut response = Response::new(404, String::new(), HashMap::new());
    let parse_result = HttpParser::parse(&stream);
    let request = match parse_result {
        Ok(request) => request,
        Err(error) => {
            println!("Error: {}", error);
            response.status(400);
            return write_response(stream, response);
        }
    };
    response.header("content-type", "text/html; charset=UTF-8");
    if routes.contains_key(&request.url) {
        response = routes[&request.url](request, response);
    }
    return write_response(stream, response);
}

fn write_response(mut stream: TcpStream, response: Response) {
    let five_seconds = Duration::new(5, 0);
    stream
        .set_write_timeout(Some(five_seconds))
        .expect("set_write_timeout call failed");
    match stream.write(response.to_http().as_bytes()) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}
