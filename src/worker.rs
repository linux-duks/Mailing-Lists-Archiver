use crate::errors;
use crate::file_utils;
use log::{Level, log_enabled};
use nntp::NNTPStream;
use std::{path::Path, thread::sleep, time::Duration};

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
    hostname: String,
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
        let nntp_stream = connect_to_nntp(format!("{}:{}", hostname.clone(), port)).unwrap();

        Worker {
            hostname,
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
                    return Err(errors::Error::NNTP(err));
                }
            }
        }
    }

    pub fn handle_group(&mut self, group_name: String) -> nntp::Result<()> {
        let read_status: ReadStatus = match file_utils::read_yaml::<ReadStatus>(
            format!(
                "{}/{}/__last_article_number",
                self.base_output_path, group_name
            )
            .as_str(),
        ) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Error reading status:  {e}");
                // attempted to read a number from the file, or fallback to 1
                let last_article_number = file_utils::try_read_number(Path::new(
                    format!(
                        "{}/{}/__last_article_number",
                        self.base_output_path, group_name
                    )
                    .as_str(),
                ))
                .unwrap_or(0);
                if last_article_number == 0 {
                    log::info!("Reading list {group_name} from mail 0");
                }

                let read_status = ReadStatus {
                    last_email: last_article_number,
                };

                // write ReadStatus
                file_utils::write_yaml(
                    format!(
                        "{}/{}/__last_article_number",
                        self.base_output_path, group_name
                    )
                    .as_str(),
                    &read_status,
                )?;

                read_status
            }
        };

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

    pub fn handle_group_range(
        &mut self,
        group_name: String,
        range: impl Iterator<Item = usize>,
    ) -> nntp::Result<()> {
        log::info!("Checking group : {group_name}");

        match self.nntp_stream.group(&group_name) {
            Ok(group) => {
                log::info!("Will start collecting mails from range for group {group}",);
                for article_number in range {
                    self.read_new_mails(group_name.clone(), article_number, article_number)?;
                }
            }
            Err(e) => {
                log::error!("failure connecting to {group_name}, error: {e}");
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
                    log::warn!(
                        "Failed reading article '{}' from '{}' : {}",
                        mail_num,
                        self.hostname,
                        e
                    );
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ReadStatus {
    pub last_email: usize,
}
