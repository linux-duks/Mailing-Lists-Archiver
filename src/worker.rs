use crate::errors;
use crate::file_utils::*;
use log::{Level, log_enabled};
use nntp::NNTPStream;
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

pub struct Worker<'a> {
    // TODO: convert to trait
    nntp_stream: &'a mut NNTPStream,
    tasklist: Arc<RwLock<BTreeMap<Instant, String>>>,
    base_output_path: String,

    reconnection_attempts_left: usize,
    needs_reconnection: bool,
}

impl Worker<'_> {
    pub fn new(
        nntp_stream: &mut NNTPStream,
        groups: Vec<String>,
        base_output_path: String,
    ) -> Worker {
        let mut tasklist: BTreeMap<Instant, String> = BTreeMap::new();

        // Schedule all groups for check in the next second
        for group in groups {
            tasklist.insert(
                Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
                group,
            );
        }
        Worker {
            nntp_stream,
            tasklist: Arc::new(RwLock::new(tasklist)),
            base_output_path,

            // TODO: make this replenish after a while without reconnection issues
            reconnection_attempts_left: 3,
            needs_reconnection: false,
        }
    }

    pub fn run(&mut self) -> crate::Result<()> {
        loop {
            // TODO: fix this loop
            if self.needs_reconnection {
                if self.reconnection_attempts_left < 1 {
                    return Err(errors::Error::NNTPReconnectionError);
                }
                self.reconnection_attempts_left -= 1;

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
                    let handler_result = self.handle_group(group_name.clone());
                    consummed.push(k);
                    if handler_result.is_err() {
                        if nntp::errors::check_network_error(handler_result.err().unwrap()) {
                            self.needs_reconnection = true;
                            break;
                        } else {
                            // TODO: check error types
                            self.needs_reconnection = true;
                            break;
                            // return Err(handler_result.err().unwrap());
                        }
                    }
                }
            }
            // removed from
            for k in consummed {
                self.tasklist.write().unwrap().remove(&k);
            }
        }
    }

    fn handle_group(&mut self, group_name: String) -> nntp::Result<()> {
        let last_article_number = read_number_or_create(Path::new(
            format!(
                "{}/{}/__last_article_number",
                self.base_output_path, group_name
            )
            .as_str(),
        ))
        .unwrap() as usize;

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
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_SUCCESS);
                        }
                        Err(e) => {
                            // if found a failure, reschedule and return error
                            // TODO: check for connection errors here ?
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
                            return Err(e);
                        }
                    };
                    // reschedule
                    self.reschedule_group(group_name.clone(), INTERVAL_AFTER_SUCCESS);
                } else {
                    log::info!(
                        "Checking group : {group_name}. Local max ID: {last_article_number}"
                    );
                    // no new emails, reschedule for next minute
                    self.reschedule_group(group_name.clone(), INTERVAL_AFTER_NO_NEWS);
                }
            }
            Err(e) => {
                log::error!("failure connecting to {group_name}, error: {e}");
                self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
            }
        }
        Ok(())
    }

    // run range does not keep track of lists, just run them once for the defined range
    pub fn run_range(&mut self, range: impl Iterator<Item = usize>) -> nntp::Result<()> {
        // let mut consummed = vec![];
        let tasklist_guard = self.tasklist.read().unwrap();

        // take all tasks, they wont repeat in this mode
        let ready_tasks: Vec<(Instant, String)> = tasklist_guard
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned().clone()))
            .collect();

        // release the lock
        drop(tasklist_guard);
        // start processing items
        // TODO: run this with more than one list ?
        let (_, group_name) = ready_tasks.first().unwrap();
        // for (k, group_name) in ready_tasks {
        self.handle_group_range(group_name.clone(), range)?;
        // consummed.push(k);
        // }
        return Ok(());
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
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_SUCCESS);
                        }
                        Err(e) => {
                            // if found a failure, reschedule and return error
                            // TODO: check for connection errors here ?
                            self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
                            return Err(e);
                        }
                    };
                }
            }
            Err(e) => {
                log::error!("failure connecting to {group_name}, error: {e}");
                self.reschedule_group(group_name.clone(), INTERVAL_AFTER_FAILURE);
            }
        }
        Ok(())
    }

    fn get_raw_article_by_number_retryable(
        &mut self,
        mail_num: isize,
        max_retries: usize,
    ) -> nntp::Result<Vec<String>> {
        let mut attempts = 0;
        let retry_delay_ms = 500;
        loop {
            match self.nntp_stream.raw_article_by_number(mail_num) {
                Ok(raw_article) => {
                    return Ok(raw_article);
                }
                Err(e) => {
                    eprintln!("Failed reading article : {}", e);
                    attempts += 1;
                    if attempts >= max_retries {
                        // Return the last error after max retries
                        return Err(e);
                    }
                    println!("Retrying in {}ms...", retry_delay_ms);
                    sleep(Duration::from_millis((retry_delay_ms * attempts) as u64));
                }
            }
        }
    }

    // read_new_mails checks for mails in an inclusive range between low and high
    fn read_new_mails(&mut self, group_name: String, low: usize, high: usize) -> nntp::Result<()> {
        // TODO: get mails by number or date (newnews command) ?

        // take the last_article_number or the "low"" result for the group
        for current_mail in low..=high {
            match self.get_raw_article_by_number_retryable(current_mail as isize, 3) {
                Ok(raw_article) => {
                    write_lines_file(
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
                    write_lines_file(
                        Path::new(
                            format!(
                                "{}/{}/__last_article_number",
                                self.base_output_path, group_name
                            )
                            .as_str(),
                        ),
                        vec![format!("{}", current_mail)],
                    )
                    .unwrap();
                }
                Err(e) => {
                    match e {
                        nntp::NNTPError::ArticleUnavailable => {
                            append_line_to_file(
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
                "{group_name} {}/{} ({}%)",
                current_mail,
                high,
                (current_mail as f64 / high as f64 * 100.0) as usize
            );
            std::thread::sleep(Duration::from_millis(10));
        }
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
