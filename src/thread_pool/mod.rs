use crate::Result;

mod naive;
pub use naive::NaiveThreadPool;

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