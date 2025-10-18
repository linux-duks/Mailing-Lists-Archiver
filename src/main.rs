#![allow(clippy::needless_return)]

use env_logger::Builder;
use log::{self, Level, LevelFilter, log_enabled};
use nntp::NNTPStream;

mod config;
mod errors;
mod file_utils;
mod range_inputs;
mod worker;
use errors::Result;

fn main() -> Result<()> {
    Builder::new()
        .filter(None, LevelFilter::Info) // Set default level to Info
        .init();

    let mut app_config = config::read_config().unwrap();

    let mut nntp_stream =
        match NNTPStream::connect((app_config.hostname.clone().unwrap(), app_config.port)) {
            Ok(stream) => stream,
            Err(e) => panic!("{}", e),
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

    let list_options = nntp_stream.list().unwrap();
    let groups = app_config
        .get_group_lists(list_options.iter().map(move |an| an.clone().name).collect())
        .unwrap();

    println!("made a selection of {} {:#?}", groups.len(), groups);

    let mut w = worker::Worker::new(&mut nntp_stream, groups, app_config.output_dir.clone());
    match app_config.get_article_range() {
        Some(range) => w.run_range(range)?,
        None => w.run()?,
    }

    let _ = nntp_stream.quit();
    Ok(())
}
