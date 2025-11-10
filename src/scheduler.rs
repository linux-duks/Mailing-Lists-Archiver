use crate::errors;
use crate::worker;
use crossbeam_channel::{bounded, unbounded};
use std::thread;
use std::{sync::Arc, time::Duration};

// intervals in seconds
const INTERVAL_BETWEEN_RESCANS: usize = 60 * 60; // 1h

pub struct Scheduler {
    hostname: String,
    port: u16,
    base_output_path: String,
    nthreds: u8,
    tasklist: Arc<Vec<String>>,
    task_channel: (
        crossbeam_channel::Sender<String>,
        crossbeam_channel::Receiver<String>,
    ),
    response_channel: (
        crossbeam_channel::Sender<worker::WorkerGroupResult>,
        crossbeam_channel::Receiver<worker::WorkerGroupResult>,
    ),
}

impl Scheduler {
    pub fn new(
        hostname: String,
        port: u16,
        base_output_path: String,
        nthreds: u8,
        groups: Vec<String>,
    ) -> Scheduler {
        let mut tasklist: Vec<String> = Vec::with_capacity(groups.len());

        // Schedule all groups for check to the next second
        for group in groups {
            tasklist.push(group.clone());
        }

        Scheduler {
            hostname,
            port,
            base_output_path,
            nthreds,
            tasklist: Arc::new(tasklist),
            task_channel: bounded::<String>(nthreds as usize),
            response_channel: unbounded::<worker::WorkerGroupResult>(),
        }
    }

    pub fn run(&mut self) -> crate::Result<()> {
        // start worker threads
        for i in 0..self.nthreds {
            log::info!("Stating worker thread {i}");

            let receiver = self.task_channel.1.clone();

            let mut worker = worker::Worker::new(
                self.hostname.clone(),
                self.port,
                self.base_output_path.clone(),
                receiver,
                self.response_channel.clone(),
            );
            // Spin up another thread
            thread::spawn(move || {
                loop {
                    match worker.comsume() {
                        Ok(_) => {
                            log::info!("Consumme finished");
                            break;
                        }
                        Err(err) => {
                            // TODO: use this to reschedule
                            log::warn!("Consummer returned an error : {err}");
                            std::thread::sleep(Duration::from_secs(1));
                        }
                    };
                }
            });
            // space out thread creation (to prevent multiple connections oppening at once)
            std::thread::sleep(Duration::from_secs(2));
        }

        loop {
            for group_name in self.tasklist.iter() {
                self.task_channel.0.send(group_name.clone()).unwrap();
            }
            // interval between checks to task list
            std::thread::sleep(Duration::from_secs(INTERVAL_BETWEEN_RESCANS as u64));
        }
    }

    // run range does not keep track of lists, just run them once for the defined range
    pub fn run_range(&mut self, range: impl Iterator<Item = usize>) -> crate::Result<()> {
        let receiver = self.task_channel.1.clone();
        let mut worker = worker::Worker::new(
            self.hostname.clone(),
            self.port,
            self.base_output_path.clone(),
            receiver,
            self.response_channel.clone(),
        );

        match self.tasklist.first() {
            Some(group_name) => {
                // TODO: map this error
                worker.handle_group_range(group_name.clone(), range)?;
                Ok(())
            }
            None => Err(errors::Error::Unknown),
        }?;

        return Ok(());
    }
}
