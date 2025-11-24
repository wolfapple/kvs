use crate::Result;

mod naive;
mod shared_queue;
mod rayon;

pub use naive::NaiveThreadPool;
pub use shared_queue::SharedQueueThreadPool;
pub use rayon::RayonThreadPool;

/// Trait for a thread pool.
pub trait ThreadPool {
    /// Create a new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    /// Spawns new job onto the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}