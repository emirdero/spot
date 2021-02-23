use crate::http_parser::HttpParser;
use crate::request::Request;
use crate::response::Response;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct ThreadPool {
    workers: Vec<Worker>,
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

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        routes: HashMap<String, fn(Request, Response) -> Response>,
        middleware: Vec<(String, fn(Request, Response) -> (Request, Response, bool))>,
    ) -> Worker {
        let thread = thread::spawn(move || 'outer: loop {
            let message = receiver.lock().unwrap().recv().unwrap();

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
    }
}
