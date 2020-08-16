use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
mod http_parser;
pub mod request;
pub mod response;

pub struct Spot {
    threads: Vec<std::thread::JoinHandle<()>>,
    stream_locks: Vec<Arc<(Mutex<bool>, Condvar)>>,
    amount_of_threads: u32,
    routes: HashMap<String, fn(request::Request, response::Response) -> response::Response>,
}

impl Spot {
    pub fn new(amount_of_threads: u32) -> Spot {
        return Spot {
            threads: Vec::new(),
            stream_locks: Vec::new(),
            amount_of_threads: amount_of_threads,
            routes: HashMap::new(),
        };
    }

    pub fn route(
        &mut self,
        path: &str,
        function: fn(request::Request, response::Response) -> response::Response,
    ) -> bool {
        if self.routes.contains_key(path) {
            println!("ERROR: Route already defined");
            return false;
        }
        self.routes.insert(path.to_owned(), function);
        return true;
    }

    pub fn bind(&mut self, ip: &str) -> Result<bool, String> {
        let listener = match TcpListener::bind(ip) {
            Ok(result) => result,
            Err(error) => {
                return Err(String::from(format!("Failed to bind to ip: {}", error)));
            }
        };

        let mut senders = Vec::new();

        for i in 0..self.amount_of_threads {
            // Communication channel
            let (sender, receiver) = mpsc::channel();
            senders.push(sender);

            let stream_lock = Arc::new((Mutex::new(true), Condvar::new()));
            let stream_lock_clone = stream_lock.clone();
            self.stream_locks.push(stream_lock);
            let routes_clone = self.routes.clone();
            let new_worker = thread::Builder::new()
                .name(format!("Spot-Worker-{}", i + 1))
                .spawn(move || loop {
                    let (lock, condvar) = &*stream_lock_clone;
                    let mut waiting = lock.lock().unwrap();
                    while *waiting {
                        waiting = condvar.wait(waiting).unwrap();
                    }
                    let mut stream: TcpStream = match receiver.recv() {
                        Ok(stream) => stream,
                        Err(error) => {
                            println!("Error: {}", error);
                            continue;
                        }
                    };
                    let result = http_parser::HttpParser::parse(&stream);
                    let request = match result {
                        Ok(request) => request,
                        Err(error) => {
                            println!("Error: {}", error);
                            continue;
                        }
                    };
                    let mut response = response::Response::new(String::new(), HashMap::new());
                    if routes_clone.contains_key(&request.url) {
                        response = routes_clone[&request.url](request, response);
                    }
                    let five_seconds = Duration::new(5, 0);
                    stream
                        .set_write_timeout(Some(five_seconds))
                        .expect("set_write_timeout call failed");
                    match stream.write(response.to_http().as_bytes()) {
                        Ok(_) => println!("Response sent"),
                        Err(e) => println!("Failed sending response: {}", e),
                    }
                    *waiting = true;
                });
            match new_worker {
                Ok(thread) => self.threads.push(thread),
                Err(error) => {
                    println!("Spot: Failed to start thread {}, ERROR: {}", i + 1, error);
                }
            }
        }

        // Stream distributor
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(error) => {
                    println!("Tcp error: {}", error);
                    return Err(String::from(format!("Tcp error: {}", error)));
                }
            };
            let mut done = false;
            let mut index = 0;
            while !done {
                index = 0;
                for stream_lock in self.stream_locks.clone() {
                    let arc = &*stream_lock;
                    let mut waiting = arc.0.try_lock();
                    let mut free = false;
                    if let Ok(ref mut mutex) = waiting {
                        **mutex = false;
                        free = true;
                    } else {
                        println!("waiting try_lock failed");
                    }

                    if free {
                        done = true;
                        arc.1.notify_one();
                        break;
                    } else {
                        index += 1;
                    }
                }
            }
            let _result = senders[index].send(stream);
        }
        // Should be unreachable
        Ok(true)
    }
}
