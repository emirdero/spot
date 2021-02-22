extern crate threadpool;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
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

    // TODO: Add middleware

    pub fn route(&mut self, path: &str, function: fn(Request, Response) -> Response) {
        let mut path_string = String::from(path);
        // Remove trailing / so that pathing is agnostic towards /example/ or /example
        let last_char = path_string.pop().unwrap();
        if last_char != '/' {
            path_string.push(last_char)
        }
        if self.routes.contains_key(&path_string) {
            println!(
                "Warning: Route defined twice ({}), using latest definition",
                path
            );
            self.routes.remove(&path_string);
        }
        self.routes.insert(path_string, function);
    }

    pub fn route_file(&mut self, path: &str) {
        if self.routes.contains_key(path) {
            println!(
                "Warning: Route defined twice ({}), using latest definition",
                path
            );
            self.routes.remove(path);
        }

        fn function(req: Request, mut res: Response) -> Response {
            if req.method == "GET" {
                let path = req.url;
                let path_split = path.split('.');
                let file_ending = path_split.last().unwrap();
                let mut file_type = "text/plain";
                if file_ending == "json" {
                    file_type = "application/json";
                } else if file_ending == "html" {
                    file_type = "text/html";
                } else if file_ending == "js" {
                    file_type = "application/javascript";
                }
                // remove first / from path
                let mut file = fs::File::open(&path[1..]).unwrap();
                let mut contents = String::new();
                let result = file.read_to_string(&mut contents);
                match result {
                    Ok(_) => {}
                    Err(error) => {
                        println!("{}", error);
                    }
                }
                res.status(200);
                res.body(contents);
                res.header("content-type", file_type);
                return res;
            } else {
                return res;
            };
        };
        // Replace Windows specific backslashes in path with forward slashes
        let result = path.replace("\\", "/");
        let route_path = format!("/{}", result);
        println!("{}", route_path);
        self.routes.insert(route_path, function);
    }

    fn add_static_files(&mut self, directory: &Path, path: &str) {
        let dir_iter = fs::read_dir(path).unwrap();

        for item in dir_iter {
            let item_uw = item.unwrap();
            let item_path = item_uw.path().into_os_string().into_string().unwrap();
            let item_metadata = item_uw.metadata().unwrap();
            if item_metadata.is_dir() {
                Spot::add_static_files(self, directory, &item_path);
            } else {
                Spot::route_file(self, &item_path);
            }
        }
    }
    pub fn public(&mut self, dir_name: &str) {
        let path = env::current_dir().unwrap();
        let root = path.join(dir_name);
        assert!(env::set_current_dir(&root).is_ok());
        let dir = env::current_dir().unwrap();
        self.add_static_files(dir.as_path(), "");
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
            println!("HTTP Parser Error: {}", error);
            response.status(400);
            return write_response(stream, response);
        }
    };
    response.header("content-type", "text/html; charset=UTF-8");
    // Remove params
    let mut request_route = String::from(request.url.split("?").next().unwrap());
    // Remove trailing / so that pathing is agnostic towards /example/ or /example
    let last_char = request_route.pop().unwrap();
    if last_char != '/' {
        request_route.push(last_char)
    }
    if routes.contains_key(&request_route) {
        response = routes[&request_route](request, response);
    }
    return write_response(stream, response);
}

fn write_response(mut stream: TcpStream, response: Response) {
    let five_seconds = Duration::new(5, 0);
    let status_code = response.status;
    stream
        .set_write_timeout(Some(five_seconds))
        .expect("set_write_timeout call failed");
    match stream.write(response.to_http().as_bytes()) {
        Ok(_) => println!("Response sent with status code {}", status_code),
        Err(e) => println!("Failed sending response: {}", e),
    }
}
