use super::ThreadPool;
use crate::Result;
use crossbeam_channel::{self, Sender, Receiver};
use std::panic;
use std::thread;

/// Type alias for a job that can be sent to the thread pool.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Enum representing messages sent to worker threads.
enum Message {
    NewJob(Job),
    Terminate,
}

/// Represents a single worker thread in the thread pool.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread that listens for jobs on the provided receiver.
    ///
    /// The worker thread will continuously try to receive messages from the `receiver`.
    /// If it receives a `NewJob`, it executes it.
    /// If it receives a `Terminate` message, it breaks its loop and exits.
    fn new(id: usize, receiver: Receiver<Message>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.recv();

                match message {
                    Ok(Message::NewJob(job)) => {
                        // Catch panics from the job to prevent the worker thread from crashing.
                        let _ = panic::catch_unwind(panic::AssertUnwindSafe(job));
                    }
                    Ok(Message::Terminate) | Err(_) => {
                        break;
                    }
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}

/// A thread pool that uses a shared queue inside.
pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool for SharedQueueThreadPool {
    /// Creates a new `SharedQueueThreadPool` with the specified number of threads.
    fn new(size: u32) -> Result<Self> {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let mut workers = Vec::with_capacity(size as usize);

        for id in 0..size {
            workers.push(Worker::new(id as usize, receiver.clone()));
        }

        Ok(SharedQueueThreadPool { workers, sender })
    }

    /// Spawns a new job onto the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender.send(Message::NewJob(job)).expect("The thread pool is dead.");
    }
}

/// Implements graceful shutdown for the thread pool.
///
/// When the `SharedQueueThreadPool` goes out of scope, its `drop` method is called.
/// This method sends `Terminate` messages to all workers and then waits for
/// each worker thread to finish its execution.
impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).ok();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
