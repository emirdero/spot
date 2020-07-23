use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time;

pub struct spot {
    threads: Vec<std::thread::JoinHandle<()>>,
    working_threads_arc: Arc<(Mutex<u32>, Condvar)>,
    amount_of_threads: u32,
}

impl spot {
    pub fn new(amount_of_threads: u32) -> Workers {
        return spot {
            threads: Vec::new(),
            working_threads_arc: Arc::new((Mutex::new(0u32), Condvar::new())),
            amount_of_threads: amount_of_threads,
        };
    }
    pub fn post(&mut self, task: fn()) {
        let working_threads = self.working_threads_arc.clone();
        let amount_of_threads_copy = self.amount_of_threads;
        let thread = thread::spawn(move || {
            let &(ref num, ref cvar) = &*working_threads;
            {
                let mut start = num.lock().unwrap();
                while *start >= amount_of_threads_copy {
                    start = cvar.wait(start).unwrap();
                }
                *start += 1;
            }
            println!("Running!");
            task();
            println!("Done!");
            let mut start = num.lock().unwrap();
            *start -= 1;
            cvar.notify_one();
        });
        self.threads.push(thread);
    }

    pub fn end(self) {
        for thread in self.threads {
            let _result = thread.join();
        }
    }
}
