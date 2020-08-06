use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
mod http_parser;
mod request;
mod response;

pub struct Spot {
    threads: Vec<std::thread::JoinHandle<()>>,
    stream_storage: Vec<Arc<Vec<Option<TcpStream>>>>,
    stream_locks: Vec<Arc<(Mutex<bool>, Condvar)>>,
    amount_of_threads: u32,
    routes: HashMap<String, fn(request::Request, response::Response) -> bool>,
}

impl Spot {
    pub fn new(amount_of_threads: u32) -> Spot {
        // Start worker threads
        let mut threads = Vec::new();
        let mut stream_storage = Vec::new();
        let mut stream_locks = Vec::new();
        for i in 0..amount_of_threads {
            let stream_lock = Arc::new((Mutex::new(true), Condvar::new()));
            let none: Option<TcpStream> = None;
            let stream_option = Arc::new(none);
            stream_locks.push(stream_lock.clone());
            stream_storage.push(stream_option.clone());
            let new_worker = thread::Builder::new()
                .name(format!("Spot-Worker-{}", i + 1))
                .spawn(move || loop {
                    let (lock, condvar) = &*stream_lock;
                    let mut waiting = lock.lock().unwrap();
                    while *waiting {
                        waiting = condvar.wait(waiting).unwrap();
                    }
                    // TODO: release lock, handle stream, set waiting to true
                });
            match new_worker {
                Ok(thread) => threads.push(thread),
                Err(error) => {
                    println!("Spot: Failed to start thread {}, ERROR: {}", i + 1, error);
                }
            }
        }
        return Spot {
            threads: threads,
            stream_storage: Vec::new(),
            stream_locks: Vec::new(),
            amount_of_threads: amount_of_threads,
            routes: HashMap::new(),
        };
    }
    pub fn route(
        &mut self,
        path: String,
        function: fn(request::Request, response::Response) -> bool,
    ) -> bool {
        if self.routes.contains_key(&path) {
            println!("ERROR: Route already defined");
            return false;
        }
        self.routes.insert(path, function);
        return true;
    }

    pub fn bind(self, ip: &str) -> Result<bool, String> {
        let listener = match TcpListener::bind(ip) {
            Ok(result) => result,
            Err(error) => {
                return Err(String::from("Failed to bind to ip"));
            }
        };
        let stream_distributor = match thread::Builder::new()
            .name(String::from("Spot-Distributor"))
            .spawn(move || {
                for stream in listener.incoming() {
                    let stream = match stream {
                        Ok(stream) => stream,
                        Err(error) => {
                            println!("Tcp error: {}", error);
                            return;
                        }
                    };
                    // TODO: assign stream to waiting worker
                }
            }) {
            Ok(handle) => handle,
            Err(error) => {
                return Err(String::from(format!(
                    "Failed to start distributor thread: {}",
                    error
                )))
            }
        };
        for thread in self.threads {
            let _result = thread.join();
        }
        let _result = stream_distributor.join();
        Ok(true)
    }
}
