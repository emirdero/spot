use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
mod request;
mod response;

pub struct Spot {
    threads: Vec<std::thread::JoinHandle<()>>,
    working_threads_arc: Arc<(Mutex<u32>, Condvar)>,
    amount_of_threads: u32,
    routes: HashMap<String, fn(request::Request, response::Response) -> bool>,
}

impl Spot {
    pub fn new(amount_of_threads: u32) -> Spot {
        // Start worker threads
        let mut threads = Vec::new();
        for i in 0..amount_of_threads {
            let new_worker = thread::Builder::new()
                .name(format!("Spot-Worker-{}", i + 1))
                .spawn(move || {
                    print!("Hello");
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
            working_threads_arc: Arc::new((Mutex::new(0u32), Condvar::new())),
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

    pub fn bind(self, ip: String) {
        for thread in self.threads {
            let _result = thread.join();
        }
    }
}
