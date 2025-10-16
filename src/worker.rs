use log::{Level, log_enabled};
use nntp::NNTPStream;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    io::{self, LineWriter, Write},
    path::Path,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    vec,
};

// intervals in seconds
const INTERVAL_AFTER_SUCCESS: usize = 60 * 60; // 1h
const INTERVAL_AFTER_NO_NEWS: usize = 60 * 60 * 2; // 2H
const INTERVAL_AFTER_FAILURE: usize = 60 * 60 * 12;

pub struct Worker<'a> {
    // TODO: convert to trait
    nntp_stream: &'a mut NNTPStream,
    tasklist: Arc<RwLock<BTreeMap<Instant, String>>>,
    base_output_path: String,
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
        }
    }

    pub fn run(&mut self) {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let mut consummed = vec![];
            {
                let tasklist_guard = self.tasklist.read().unwrap();

                let ready_tasks: Vec<(Instant, String)> = tasklist_guard
                    .iter()
                    .take_while(|(k, _)| **k <= Instant::now())
                    .map(|(k, v)| (k.to_owned(), v.to_owned().clone()))
                    .collect();

                // release the lock
                drop(tasklist_guard);

                // start processing items
                for (k, group_name) in ready_tasks {
                    self.handle_group(group_name.clone());
                    consummed.push(k);
                }
            }
            // removed from
            for k in consummed {
                self.tasklist.write().unwrap().remove(&k);
            }
        }
    }

    fn handle_group(&mut self, group_name: String) {
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
                    self.read_new_mails(
                        group_name.clone(),
                        last_article_number,
                        group.high as usize,
                        group.low as usize,
                    );
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
    }

    fn read_new_mails(
        &mut self,
        group_name: String,
        last_article_number: usize,
        high: usize,
        low: usize,
    ) {
        // TODO: get mails by number or date (newnews command) ?

        // take the last_article_number or the "low"" result for the group
        for current_mail in last_article_number.max(low)..high {
            match self
                .nntp_stream
                .raw_article_by_number(current_mail as isize)
            {
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
                Err(e) => panic!("Error reading mail {}", e),
            }

            log::info!(
                "{group_name} {}/{} ({}%)",
                current_mail,
                high,
                (current_mail as f64 / high as f64 * 100.0) as usize
            );
            std::thread::sleep(Duration::from_millis(10));
        }
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

fn write_lines_file(path: &Path, lines: Vec<String>) -> io::Result<()> {
    // Create or open a file for writing
    let file = File::create(path)?;
    let mut file = LineWriter::new(file);

    lines
        .iter()
        .map(|line| write!(file, "{}", line.as_str()).expect("Cannot write to file"))
        .collect::<Vec<_>>();

    file.flush()?;

    log::debug!("file written {}", path.to_str().unwrap());

    Ok(())
}

/// Reads a number from a file.
///
/// If the file or its parent directories do not exist, it creates them
/// and initializes the file with the content "1".
///
/// # Arguments
/// * `path` - A reference to the path of the file to read.
/// # Returns
///
/// * `Result<usize, Box<dyn Error>>` - The parsed number on success, or a boxed error on failure.
fn read_number_or_create(path: &Path) -> Result<usize, Box<dyn Error>> {
    // Attempt to read the file's content into a string.
    match fs::read_to_string(path) {
        // The file was read successfully
        Ok(content) => {
            log::info!("Successfully read file: {}", path.display());
            // Trim whitespace and parse the content into an usize integer.
            // If parsing fails, return a custom error.
            let number = content.trim().parse::<usize>().map_err(|e| {
                // TODO: map to error type
                format!(
                    "Could not parse file content '{}' as a number: {}",
                    content.trim(),
                    e
                )
            })?;
            Ok(number)
        }
        //  An error occurred while reading
        Err(error) => {
            // Check if the error was "File Not Found".
            if error.kind() == io::ErrorKind::NotFound {
                log::info!("File not found at {}. Creating it...", path.display());

                // Get the parent directory of the specified path.
                if let Some(parent_dir) = path.parent() {
                    // Create the full directory path if it doesn't exist.
                    // `create_dir_all` is convenient as it won't error if the path already exists.
                    fs::create_dir_all(parent_dir)?;
                    log::info!("Ensured directory exists: {}", parent_dir.display());
                }

                // Create the file and write the default value "1" to it.
                fs::write(path, "1")?;
                log::info!("Created and initialized file with '1'.");

                // Since we just created it with "1", we can return 1.
                Ok(1)
            } else {
                Err(Box::new(error))
            }
        }
    }
}
