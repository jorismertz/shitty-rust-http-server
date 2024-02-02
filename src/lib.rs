use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub enum Status {
    Ok,
    Created,
    Accepted,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl Status {
    fn status_code(&self) -> i32 {
        return match self {
            Self::Ok => 200,
            Self::Created => 201,
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::NotFound => 404,
            Self::InternalServerError => 500,
        };
    }
    fn response_string(&self) -> String {
        return format!("HTTP/1.1 {} {}", self.status_code(), self);
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = format!("{:?}", self);

        let mut result: Vec<char> = Vec::new();

        for (i, char) in as_string.chars().enumerate() {
            if char.is_uppercase() && i != 0 {
                result.push(char::from_str(" ").unwrap());
                result.push(char);
                continue;
            }
            result.push(char)
        }

        let as_string: String = result.iter().collect();

        write!(f, "{}", as_string.to_uppercase()).unwrap();
        return Ok(());
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        return Worker {
            id,
            thread: Some(thread),
        };
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

