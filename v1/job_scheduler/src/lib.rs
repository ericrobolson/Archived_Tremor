use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;

pub type JobFunction = Box<dyn FnOnce() + Send + 'static>;

/// The messages a job thread may receive.
enum Jobs {
    /// Execute the given closure.
    Work(JobFunction),
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
    queued_jobs: Vec<JobFunction>,
    job_receiver: Receiver<JobFunction>,
    job_sender: Sender<JobFunction>,
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

        let (job_sender, job_receiver) = mpsc::channel();

        Self {
            job_processors,
            queued_jobs: vec![],
            job_sender,
            job_receiver,
        }
    }

    /// Queues a job to execute.
    pub fn queue(&mut self, job: impl FnOnce() + Send + 'static) {
        self.queued_jobs.push(Box::new(job));
    }

    /// A sender for the job queue. May be passed along threads.
    pub fn inbox(&self) -> Sender<JobFunction> {
        self.job_sender.clone()
    }

    /// Check for finished jobs and queue outstanding ones. Returns how many jobs remain.
    pub fn process(&mut self) -> usize {
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

        // Check inbox for queued jobs from other threads
        for job in self.job_receiver.try_recv() {
            self.queue(job);
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

        // Return outstanding jobs
        self.queued_jobs.len() + jobs_in_progress
    }

    /// The loop for a job. If no work is found, will exponentially increase the sleep time so as not to overload CPU usage.
    fn job(receiver: Receiver<Jobs>, schedule_sender: Sender<JobResponse>) {
        use std::time::Duration;
        const MAX_SLEEP: Duration = Duration::from_millis(5);
        const MIN_SLEEP: Duration = Duration::from_micros(100);
        let mut consecutive_sleeps = 0;

        loop {
            match receiver.try_recv() {
                Ok(job) => match job {
                    Jobs::Work(work) => {
                        // Do the work and then let the scheduler know that it was completed.
                        consecutive_sleeps = 0;
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
                            // Do nothing and sleep the thread.
                            consecutive_sleeps += 1;
                            let sleep = {
                                let sleep = MIN_SLEEP * consecutive_sleeps * consecutive_sleeps;
                                if sleep > MAX_SLEEP {
                                    MAX_SLEEP
                                } else {
                                    sleep
                                }
                            };

                            std::thread::sleep(sleep);
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
