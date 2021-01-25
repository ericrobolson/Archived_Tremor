use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;

/// The messages a job thread may receive.
enum Jobs {
    /// Execute the given closure.
    Work(Box<dyn FnOnce() + Send + 'static>),
    /// Shut down the thread.
    Shutdown,
}

/// Messages a job thread may send to the scheduler.
enum JobResponse {
    Err,
    Completed,
}

/// Data surrounding a job processor thread.
struct JobProcessor {
    handle: thread::JoinHandle<()>,
    sender: Sender<Jobs>,
    inbox: Receiver<JobResponse>,
    job_in_progress: bool,
}

/// A job scheduler that queues up work and executes on multiple threads.
/// ## Example usage
/// ```
///  let mut scheduler = JobScheduler::new(10);
///
///  for i in 0..num_loops {
///     let work = move || {
///         expensive_calculation();
///     };
///
///     scheduler.queue(work);
///   }
///
///   scheduler.block();
/// ```
pub struct JobScheduler {
    job_processors: Vec<JobProcessor>,
    queued_jobs: Vec<Box<dyn FnOnce() + Send + 'static>>,
}

impl JobScheduler {
    /// Creates a new job scheduler with the specified thread count.
    pub fn new(num_threads: u32) -> Self {
        let mut job_processors = vec![];

        for _thread_id in 0..num_threads {
            let (job_thread_sender, job_thread_inbox): (Sender<Jobs>, Receiver<Jobs>) =
                mpsc::channel();
            let (scheduler_sender, scheduler_inbox): (Sender<JobResponse>, Receiver<JobResponse>) =
                mpsc::channel();

            let join_handle = thread::spawn(move || {
                Self::job(job_thread_inbox, scheduler_sender);
            });

            job_processors.push(JobProcessor {
                inbox: scheduler_inbox,
                handle: join_handle,
                sender: job_thread_sender,
                job_in_progress: false,
            });
        }

        Self {
            job_processors,
            queued_jobs: vec![],
        }
    }

    /// Queues a job to execute.
    pub fn queue(&mut self, job: impl FnOnce() + Send + 'static) {
        self.queued_jobs.push(Box::new(job));
    }

    /// Process all queued jobs. Blocks until they all complete.
    pub fn block(&mut self) {
        let mut stop_processing = false;

        while !stop_processing {
            let mut disconnected_processors = vec![];

            // Process incoming messages
            for (processor_index, processor) in self.job_processors.iter_mut().enumerate() {
                let mut receiving = true;
                while receiving {
                    match processor.inbox.try_recv() {
                        Ok(message) => {
                            processor.job_in_progress = false;
                            match message {
                                JobResponse::Err => {
                                    unimplemented!("TODO: recieved a job error. How to handle?");
                                }
                                JobResponse::Completed => {
                                    processor.job_in_progress = false;
                                }
                            }
                        }
                        Err(err) => {
                            match err {
                                TryRecvError::Empty => {
                                    // Do nothing
                                    processor.job_in_progress = false;
                                }
                                TryRecvError::Disconnected => {
                                    disconnected_processors.push(processor_index);
                                }
                            }

                            receiving = false;
                        }
                    }
                }
            }

            // Remove disconnected processors. Since they're processed in order, remove the processor starting from the back of the list.
            for disconnected_processor in disconnected_processors.iter().rev() {
                self.job_processors.remove(*disconnected_processor);
            }

            // Send queued jobs out
            let mut jobs_in_progress = 0;
            for processor in self.job_processors.iter_mut() {
                if processor.job_in_progress == false {
                    // Queue up a job
                    match self.queued_jobs.pop() {
                        Some(job) => {
                            processor.job_in_progress = true;
                            jobs_in_progress += 1;
                            match processor.sender.send(Jobs::Work(job)) {
                                Ok(_) => {}
                                Err(e) => {
                                    panic!("Error sending job!: {:?}", e);
                                }
                            }
                        }
                        None => {}
                    }
                } else {
                    jobs_in_progress += 1;
                }
            }

            // Check if all jobs are processed.
            stop_processing = self.queued_jobs.is_empty() && jobs_in_progress == 0;
        }
    }

    /// The loop for a job.
    fn job(receiver: Receiver<Jobs>, schedule_sender: Sender<JobResponse>) {
        loop {
            match receiver.try_recv() {
                Ok(job) => match job {
                    Jobs::Work(work) => {
                        // Do the work and then let the scheduler know that it was completed.
                        work();
                        schedule_sender.send(JobResponse::Completed).unwrap();
                    }
                    Jobs::Shutdown => {
                        return;
                    }
                },
                Err(error) => {
                    match error {
                        TryRecvError::Empty => {
                            // Do nothing
                        }
                        TryRecvError::Disconnected => {
                            println!("This job failed!");
                            schedule_sender.send(JobResponse::Err).unwrap();
                            return;
                        }
                    }
                }
            }
        }
    }
}

impl Drop for JobScheduler {
    fn drop(&mut self) {
        // Shut down all threads.
        for processor in &self.job_processors {
            match processor.sender.send(Jobs::Shutdown) {
                Ok(_) => {}
                Err(_) => {}
            }
        }

        // Join them all.
        while let Some(processor) = self.job_processors.pop() {
            match processor.handle.join() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
