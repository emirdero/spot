use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::path::Path;

pub mod http_parser;
pub mod request;
pub mod response;
pub mod threadpool;
use request::Request;
use response::Response;
use threadpool::ThreadPool;

pub struct Spot {
    amount_of_threads: usize,
    routes: HashMap<String, fn(Request, Response) -> Response>,
    middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
}

impl Spot {
    pub fn new(amount_of_threads: usize) -> Spot {
        return Spot {
            amount_of_threads: amount_of_threads,
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
        if last_char != '/' || path_string.len() == 0 {
            path_string.push(last_char)
        }
        self.middleware.push((path_string, function));
    }

    pub fn route(&mut self, path: &str, function: fn(Request, Response) -> Response) {
        let mut path_string = String::from(path);
        // Remove trailing / so that pathing is agnostic towards /example/ or /example
        let last_char = path_string.pop().unwrap();
        if last_char != '/' || path_string.len() == 0 {
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
        let new_root_dir = path.join(dir_name);
        // Set the specified directory as the root when reading files
        assert!(env::set_current_dir(&new_root_dir).is_ok());
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

        // clone routes and middleware
        let routes_clone = self.routes.clone();
        let middleware_clone = self.middleware.clone();

        // Create threadpool
        let pool = ThreadPool::new(self.amount_of_threads, routes_clone, middleware_clone);

        println!("Spot server listening on: http://{}", ip);
        for stream in listener.incoming() {
            match stream {
                Ok(stream_uw) => {
                    pool.execute(stream_uw);
                }
                Err(error) => println!("{}", error),
            }
        }
        return String::from("Shutting down.");
    }
}
