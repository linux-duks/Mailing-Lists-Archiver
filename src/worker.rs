use crate::errors;
use crate::file_utils;
use crossbeam_channel::{TryRecvError, bounded, unbounded};
use log::{Level, log_enabled};
use nntp::NNTPStream;
use std::thread;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, Instant},
    vec,
};

// intervals in seconds
const INTERVAL_AFTER_SUCCESS: usize = 60 * 60; // 1h
const INTERVAL_AFTER_NO_NEWS: usize = 60 * 60 * 2; // 2H
const INTERVAL_AFTER_FAILURE: usize = 60 * 30; // 30min

pub fn connect_to_nntp(address: String) -> nntp::Result<NNTPStream> {
    let mut nntp_stream = match NNTPStream::connect(address) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(e);
        }
    };

    match nntp_stream.capabilities() {
        Ok(lines) => {
            if log_enabled!(Level::Debug) {
                log::debug!("server capabilities");
                for line in lines.iter() {
                    log::debug!("{}", line);
                }
            }
        }
        Err(e) => log::error!("Failed checking server capabilities: {}", e),
    }
    return Ok(nntp_stream);
}

pub struct Worker {
    nntp_stream: NNTPStream,
    base_output_path: String,
    needs_reconnection: bool,
    receiver: crossbeam_channel::Receiver<String>,
    response_channel: (
        crossbeam_channel::Sender<WorkerGroupResult>,
        crossbeam_channel::Receiver<WorkerGroupResult>,
    ),
}

impl Worker {
    pub fn new(
        hostname: String,
        port: u16,
        base_output_path: String,
        receiver: crossbeam_channel::Receiver<String>,
        response_channel: (
            crossbeam_channel::Sender<WorkerGroupResult>,
            crossbeam_channel::Receiver<WorkerGroupResult>,
        ),
    ) -> Worker {
        let nntp_stream = connect_to_nntp(format!("{}:{}", hostname, port)).unwrap();

        Worker {
            base_output_path,
            nntp_stream,
            needs_reconnection: false,
            receiver,
            response_channel,
        }
    }

    pub fn comsume(&mut self) -> crate::Result<()> {
        loop {
            if self.needs_reconnection {
                log::debug!("Will attempt a reconnection soon");
                // wait  a minute before trying to reconnect
                std::thread::sleep(Duration::from_secs(60));

                log::info!("Will attempt a reconnection");
                match self.nntp_stream.re_connect() {
                    Ok(_) => self.needs_reconnection = false,
                    Err(e) => {
                        log::error!("attempted reconnection and failed with error {e}");
                        return Err(errors::Error::NNTP(e));
                    }
                }
            } else {
                // interval between checks to task list
                std::thread::sleep(Duration::from_secs(5));
            }
            log::info!("Worker Reading new group from channel");
            let group_name = self.receiver.recv().unwrap();
            let handler_result = self.handle_group(group_name.clone());
            if handler_result.is_err() {
                let err = handler_result.unwrap_err();
                if nntp::errors::check_network_error(&err) {
                    self.needs_reconnection = true;
                } else {
                    log::error!(
                        "Consummer failed while processing {group_name} with error {}",
                        &err
                    );
                    // break;
                    return Err(errors::Error::NNTP(err));
                }
            }
        }
    }

    fn handle_group(&mut self, group_name: String) -> nntp::Result<()> {
        let read_status: ReadStatus = file_utils::read_yaml::<ReadStatus>(
            format!(
                "{}/{}/__last_article_number",
                self.base_output_path, group_name
            )
            .as_str(),
        )
        // fallback to old format
        // TODO: remove after out files are all in new format
        .unwrap_or({
            let last_article_number = file_utils::read_number_or_create(Path::new(
                format!(
                    "{}/{}/__last_article_number",
                    self.base_output_path, group_name
                )
                .as_str(),
            ))
            .unwrap() as usize;

            ReadStatus {
                last_email: last_article_number,
                timestamp: chrono::Utc::now(),
            }
        });

        let last_article_number = read_status.last_email;

        log::info!("Checking group : {group_name}. Local max ID: {last_article_number}");

        match self.nntp_stream.group(&group_name) {
            Ok(group) => {
                log::info!(
                    "Remote max for {} is {}, local is {}",
                    group_name,
                    group.high,
                    last_article_number
                );

                if last_article_number < group.high as usize {
                    log::info!("Reading emails for group : {group_name}.");
                    // this call may return an IO error,
                    match self.read_new_mails(
                        group_name.clone(),
                        last_article_number.max(group.low as usize),
                        group.high as usize,
                    ) {
                        Ok(_) => {
                            // if successfull, reschedule
                            self.response_channel
                                .0
                                .send(WorkerGroupResult::Ok(group_name.clone()))
                                .unwrap();
                        }
                        Err(e) => {
                            // if found a failure, reschedule and return error
                            // TODO: check for connection errors here ?
                            self.response_channel
                                .0
                                .send(WorkerGroupResult::Failed(group_name.clone()))
                                .unwrap();
                            return Err(e);
                        }
                    };
                    // reschedule
                    self.response_channel
                        .0
                        .send(WorkerGroupResult::Ok(group_name.clone()))
                        .unwrap();
                } else {
                    log::info!(
                        "Checking group : {group_name}. Local max ID: {last_article_number}"
                    );
                    // no new emails, reschedule for next minute
                    self.response_channel
                        .0
                        .send(WorkerGroupResult::NoNews(group_name.clone()))
                        .unwrap();
                }
            }
            Err(e) => {
                log::error!("failure connecting to {group_name}, error: {e}");
                self.response_channel
                    .0
                    .send(WorkerGroupResult::Failed(group_name.clone()))
                    .unwrap();
            }
        }
        Ok(())
    }

    fn handle_group_range(
        &mut self,
        group_name: String,
        range: impl Iterator<Item = usize>,
    ) -> nntp::Result<()> {
        log::info!("Checking group : {group_name}");

        match self.nntp_stream.group(&group_name) {
            Ok(group) => {
                log::info!("Will start collecting mails from range for group {group}",);
                for article_number in range {
                    // this call may return an IO error,
                    match self.read_new_mails(group_name.clone(), article_number, article_number) {
                        Ok(_) => {
                            // if successfull, reschedule
                            // self.reschedule_group(group_name.clone(), INTERVAL_AFTER_SUCCESS);
                        }
                        Err(e) => {
                            // if found a failure, reschedule and return error
                            // TODO: check for connection errors here ?
                            // self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
                            return Err(e);
                        }
                    };
                }
            }
            Err(e) => {
                log::error!("failure connecting to {group_name}, error: {e}");
                // self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
            }
        }
        Ok(())
    }

    // read_new_mails checks for mails in an inclusive range between low and high
    fn read_new_mails(&mut self, group_name: String, low: usize, high: usize) -> nntp::Result<()> {
        // TODO: get mails by number or date (newnews command) ?

        // take the last_article_number or the "low"" result for the group
        for current_mail in low..=high {
            match self.get_raw_article_by_number_retryable(current_mail as isize, 3) {
                Ok(raw_article) => {
                    file_utils::write_lines_file(
                        Path::new(
                            format!(
                                "{}/{}/{}.eml",
                                self.base_output_path, group_name, current_mail
                            )
                            .as_str(),
                        ),
                        raw_article,
                    )
                    .unwrap();

                    // write ReadStatus
                    file_utils::write_yaml(
                        format!(
                            "{}/{}/__last_article_number",
                            self.base_output_path, group_name
                        )
                        .as_str(),
                        &ReadStatus {
                            last_email: current_mail,
                            timestamp: chrono::Utc::now(),
                        },
                    )?;
                }
                Err(e) => {
                    match e {
                        nntp::NNTPError::ArticleUnavailable => {
                            file_utils::append_line_to_file(
                                Path::new(
                                    format!("{}/{}/__errors", self.base_output_path, group_name)
                                        .as_str(),
                                ),
                                format!("{current_mail},{e}").as_str(),
                            )
                            .unwrap();
                            log::warn!("Email with number {current_mail} unavailable");
                        }
                        _ => return Err(e),
                    }
                    // // TODO: should the program singnal a need to reconnect here or upstream ?
                    // return Err(e);
                }
            }

            log::info!(
                "{group_name} {}/{} ({:.2}%)",
                current_mail,
                high,
                (current_mail as f64 / high as f64 * 100.0)
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        return Ok(());
    }

    fn get_raw_article_by_number_retryable(
        &mut self,
        mail_num: isize,
        max_retries: usize,
    ) -> nntp::Result<Vec<String>> {
        let mut attempts = 0;
        let retry_delay_ms = 600;
        loop {
            match self.nntp_stream.raw_article_by_number(mail_num) {
                Ok(raw_article) => {
                    return Ok(raw_article);
                }
                Err(e) => {
                    log::warn!("Failed reading article : {}", e);
                    attempts += 1;
                    if attempts > max_retries {
                        // Return the last error after max retries
                        return Err(e);
                    }
                    log::warn!("Retrying in {}ms...", (retry_delay_ms * (attempts + 1)));
                    sleep(Duration::from_millis(
                        (retry_delay_ms * (attempts + 1)) as u64,
                    ));
                }
            }
        }
    }
}

pub enum WorkerGroupResult {
    Ok(String),
    Failed(String),
    NoNews(String),
}

pub struct Scheduler {
    hostname: String,
    port: u16,
    base_output_path: String,
    nthreds: u8,
    tasklist: Arc<RwLock<BTreeMap<Instant, String>>>,
    task_channel: (
        crossbeam_channel::Sender<String>,
        crossbeam_channel::Receiver<String>,
    ),
    response_channel: (
        crossbeam_channel::Sender<WorkerGroupResult>,
        crossbeam_channel::Receiver<WorkerGroupResult>,
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
        let mut tasklist: BTreeMap<Instant, String> = BTreeMap::new();

        // Schedule all groups for check to the next second
        for group in groups {
            tasklist.insert(
                Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
                group,
            );
        }

        Scheduler {
            hostname,
            port,
            base_output_path,
            nthreds,
            tasklist: Arc::new(RwLock::new(tasklist)),
            task_channel: bounded::<String>(nthreds as usize),
            response_channel: unbounded::<WorkerGroupResult>(),
        }
    }

    pub fn run(&mut self) -> crate::Result<()> {
        // start worker threads
        for i in 0..self.nthreds {
            log::info!("Stating worker thread {i}");

            let receiver = self.task_channel.1.clone();

            let mut worker = Worker::new(
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
                            log::warn!("Consumme returned an error : {err}");
                        }
                    };
                }
            });
        }

        loop {
            // interval between checks to task list
            std::thread::sleep(Duration::from_secs(5));
            let mut consummed = vec![];
            {
                let tasklist_guard = self.tasklist.read().unwrap();

                // filter only tasks ready to be run
                let ready_tasks: Vec<(Instant, String)> = tasklist_guard
                    .iter()
                    .take_while(|(k, _)| **k <= Instant::now())
                    .map(|(k, v)| (k.to_owned(), v.to_owned().clone()))
                    .collect();

                // release the lock
                drop(tasklist_guard);

                // start processing items
                for (k, group_name) in ready_tasks {
                    // this call should block because of the size of the channel
                    self.task_channel.0.send(group_name).unwrap();
                    consummed.push(k);
                }
            }
            // removed from btree
            for k in consummed {
                self.tasklist.write().unwrap().remove(&k);
            }

            loop {
                // check groups returned by workers, and reschedule them
                match self.response_channel.1.try_recv() {
                    Ok(msg) => match msg {
                        WorkerGroupResult::Ok(group_name) => {
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_SUCCESS);
                        }
                        WorkerGroupResult::Failed(group_name) => {
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
                        }
                        WorkerGroupResult::NoNews(group_name) => {
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_NO_NEWS);
                        }
                    },
                    Err(TryRecvError::Empty) => {
                        // No message available, perform other tasks
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        // Sender has disconnected and channel is empty
                        log::info!("Sender disconnected. Exiting loop.");
                        return Ok(());
                    }
                }
            }
        }
    }

    // run range does not keep track of lists, just run them once for the defined range
    pub fn run_range(&mut self, range: impl Iterator<Item = usize>) -> crate::Result<()> {
        let tasklist_guard = self.tasklist.read().unwrap();

        // take all tasks, they wont repeat in this mode
        let ready_tasks: Vec<(Instant, String)> = tasklist_guard
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned().clone()))
            .collect();

        // release the lock
        drop(tasklist_guard);

        let receiver = self.task_channel.1.clone();
        let mut worker = Worker::new(
            self.hostname.clone(),
            self.port,
            self.base_output_path.clone(),
            receiver,
            self.response_channel.clone(),
        );
        // start processing items
        // TODO: run this with more than one list ?
        let (_, group_name) = ready_tasks.first().unwrap();
        // for (k, group_name) in ready_tasks {
        worker.handle_group_range(group_name.clone(), range)?;
        // consummed.push(k);
        // }
        return Ok(());
    }

    fn reschedule_group(&mut self, group_name: String, seconds_next_check: usize) {
        let run_at = Instant::now()
            .checked_add(Duration::from_secs(seconds_next_check as u64))
            .unwrap();

        if log_enabled!(Level::Debug) {
            log::debug!("group scheduled to {} +{}s", group_name, seconds_next_check);
        }
        // re add the task
        let mut tasklist_guard = self.tasklist.write().unwrap();

        tasklist_guard.insert(run_at, group_name);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ReadStatus {
    pub last_email: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
