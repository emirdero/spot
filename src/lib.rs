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
    middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
}

impl Spot {
    pub fn new(amount_of_threads: usize) -> Spot {
        return Spot {
            pool: ThreadPool::new(amount_of_threads),
            routes: HashMap::new(),
            middleware: Vec::new(),
        };
    }

    pub fn middle(
        &mut self,
        path: &str,
        function: fn(Request, Response) -> (Request, Response, bool),
    ) {
        let mut path_string = String::from(path);
        // Remove trailing / so that pathing is agnostic towards /example/ or /example
        let last_char = path_string.pop().unwrap();
        if last_char != '/' {
            path_string.push(last_char)
        }
        self.middleware.push((path_string, function));
    }

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
        fn function(req: Request, mut res: Response) -> Response {
            if req.method == "GET" {
                let path = req.url;
                let path_split = path.split('.');
                let file_ending = path_split.last().unwrap();
                let supported_types = [
                    ("html", "text/html"),
                    ("css", "text/css"),
                    ("json", "application/json"),
                    ("js", "application/javascript"),
                    ("zip", "application/zip"),
                    ("csv", "text/csv"),
                    ("xml", "text/xml"),
                    ("ico", "image/x-icon"),
                    ("jpg", "image/jpeg"),
                    ("jpeg", "image/jpeg"),
                    ("png", "image/png"),
                    ("gif", "image/gif"),
                    ("mp3", "audio/mpeg"),
                    ("mp4", "video/mp4"),
                    ("webm", "video/webm"),
                ];
                let mut file_type = "text/plain";
                for supported_type in supported_types.iter() {
                    if supported_type.0 == file_ending {
                        file_type = supported_type.1;
                    }
                }
                // remove first / from path and read metadata then file
                match fs::metadata(&path[1..]) {
                    Ok(metadata) => {
                        let mut contents = vec![0; metadata.len() as usize];
                        match fs::File::open(&path[1..]) {
                            Ok(mut file) => {
                                let result = file.read(&mut contents);
                                match result {
                                    Ok(_) => {
                                        res.status(200);
                                        res.body_bytes(contents);
                                        res.header("content-type", file_type);
                                    }
                                    Err(error) => {
                                        println!("{}", error);
                                        res.status(500);
                                    }
                                }
                            }
                            Err(error) => {
                                println!("{}", error);
                                res.status(500);
                            }
                        }
                    }
                    Err(error) => {
                        println!("{}", error);
                        res.status(500);
                    }
                }
            }
            return res;
        };
        // Replace Windows specific backslashes in path with forward slashes
        let result = path.replace("\\", "/");
        let route_path = format!("/{}", result);
        Spot::route(self, &route_path, function);
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
        // Sort middleware by length
        self.middleware.sort_by(|a, b| a.0.len().cmp(&b.0.len()));

        for mid in &self.middleware {
            println!("{}", mid.0);
        }

        println!("Spot server listening on: http://{}", ip);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let routes_clone = self.routes.clone();
            let middleware_clone = self.middleware.clone();
            self.pool.execute(|| {
                handle_request(stream, routes_clone, middleware_clone);
            });
        }
        return String::from("Shutting down.");
    }
}

fn handle_request(
    stream: TcpStream,
    routes: HashMap<String, fn(Request, Response) -> Response>,
    middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
) {
    let mut response = Response::new(404, Vec::new(), HashMap::new());
    let parse_result = HttpParser::parse(&stream);
    let mut request = match parse_result {
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
        // Route through middleware
        for mid in middleware {
            if mid.0.len() > request_route.len() {
                break;
            };
            if mid.0 == request_route[..mid.0.len()] {
                let answer = mid.1(request, response);
                response = answer.1;
                // If the middleware rejects the request we return the response
                if !answer.2 {
                    return write_response(stream, response);
                }
                request = answer.0;
            }
        }
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
    match stream.write(&response.to_http()) {
        Ok(_) => println!("Response sent with status code {}", status_code),
        Err(e) => println!("Failed sending response: {}", e),
    }
}
