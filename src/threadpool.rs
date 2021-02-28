use crate::http_parser::HttpParser;
use crate::request::Request;
use crate::response::Response;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ThreadPool {
    // A vector containing worker threads equal to the amount specified in new()
    workers: Vec<Worker>,
    // A channel for forwarding jobs to the worker threads
    sender: mpsc::Sender<Message>,
}

enum Message {
    NewJob(TcpStream),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(
        size: usize,
        routes: HashMap<String, fn(Request, Response) -> Response>,
        middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
    ) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let routes_clone = routes.clone();
            let middleware_clone = middleware.clone();
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                routes_clone,
                middleware_clone,
            ));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute(&self, stream: TcpStream) {
        match self.sender.send(Message::NewJob(stream)) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error);
            }
        };
    }
}

/// Stops the threads gracefully, meaning that they finish their current tasks and then end themselves. Currently not in use
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            match self.sender.send(Message::Terminate) {
                Ok(_) => {}
                Err(error) => println!("{}", error),
            }
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => {}
                    Err(_error) => println!("Failed to join thread"),
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The worker stores clones of the routes and middleware hashmaps for handling requests. The reveiver is used to forward jobs into the thread.
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        routes: HashMap<String, fn(Request, Response) -> Response>,
        middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
    ) -> Worker {
        let thread = thread::spawn(move || 'outer: loop {
            // Receive message from main thread
            let lock = match receiver.lock() {
                Ok(lock) => lock,
                Err(error) => {
                    println!("{}", error);
                    continue 'outer;
                }
            };
            let message = match lock.recv() {
                Ok(message) => message,
                Err(error) => {
                    println!("{}", error);
                    continue 'outer;
                }
            };

            // Handle job
            match message {
                Message::NewJob(stream) => {
                    let mut response = Response::new(404, Vec::new(), HashMap::new());
                    let parse_result = HttpParser::parse(&stream);
                    let mut request = match parse_result {
                        Ok(request) => request,
                        Err(error) => {
                            println!("HTTP Parser Error: {}", error);
                            response.status(400);
                            write_response(stream, response);
                            continue 'outer; // Skip to next iteration
                        }
                    };
                    // Remove params
                    let request_wo_params = match request.url.split("?").next() {
                        Some(url) => url,
                        None => {
                            response.status(400);
                            write_response(stream, response);
                            continue 'outer; // Skip to next iteration
                        }
                    };
                    let mut request_route = String::from(request_wo_params);
                    // Remove trailing / so that pathing is agnostic towards /example/ or /example
                    let last_char = match request_route.pop() {
                        Some(character) => character,
                        None => {
                            response.status(500);
                            response.body("failed to parse http");
                            write_response(stream, response);
                            continue 'outer;
                        }
                    };
                    if last_char != '/' || request_route.len() == 0 {
                        request_route.push(last_char)
                    }
                    if routes.contains_key(&request_route) {
                        // Route through middleware
                        for mid in &middleware {
                            if mid.0.len() > request_route.len() {
                                break;
                            };
                            if mid.0 == request_route[..mid.0.len()] {
                                let answer = mid.1(request, response);
                                response = answer.1;
                                // If the middleware rejects the request we return the response
                                if !answer.2 {
                                    write_response(stream, response);
                                    continue 'outer;
                                }
                                request = answer.0;
                            }
                        }
                        response = routes[&request_route](request, response);
                    }
                    write_response(stream, response);
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        return Worker {
            id,
            thread: Some(thread),
        };
        // Writes a tcp response to the client
        fn write_response(mut stream: TcpStream, response: Response) {
            let five_seconds = Duration::new(5, 0);
            stream
                .set_write_timeout(Some(five_seconds))
                .expect("set_write_timeout call failed");
            match stream.write(&response.to_http()) {
                Ok(_) => {}
                Err(e) => println!("Failed sending response: {}", e),
            }
        }
    }
}
