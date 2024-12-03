use tokio::runtime::Runtime;
use std::sync::Arc;
use std::future::Future;
use crossbeam_queue::ArrayQueue;

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    task_queue: Arc<ArrayQueue<Task>>,
    runtime: Runtime,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let runtime = Runtime::new().expect("Failed to create runtime");

        let task_queue = Arc::new(ArrayQueue::new(10_000));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&task_queue),
                runtime.handle().clone(),
            ));
        }

        Self {
            workers,
            task_queue,
            runtime,
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Task::new(future);
        self.task_queue.push(task).expect("queue full");
    }
}

#[derive(Debug)]
struct Task;

struct Worker;

impl Task {
    fn new<F>(_future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task
    }
}

impl Worker {
    fn new(_id: usize, _task_queue: Arc<ArrayQueue<Task>>, _handle: tokio::runtime::Handle) -> Self {
        Worker
    }
}
