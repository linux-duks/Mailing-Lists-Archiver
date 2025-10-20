#![allow(clippy::needless_return)]

use env_logger::{Builder, Env};
use log::{self, LevelFilter};

mod config;
mod errors;
mod file_utils;
mod range_inputs;
mod worker;
use errors::Result;

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let mut app_config = config::read_config().unwrap();

    let mut nntp_stream =
        worker::connect_to_nntp(app_config.hostname.clone().unwrap(), app_config.port)?;

    let list_options = nntp_stream.list().unwrap();
    let groups = app_config
        .get_group_lists(list_options.iter().map(move |an| an.clone().name).collect())
        .unwrap();

    // close initial connection to nntp server
    let _ = nntp_stream.quit();

    println!("made a selection of {} {:#?}", groups.len(), groups);

    let mut w = worker::Worker::new(&app_config, groups);
    match app_config.get_article_range() {
        Some(range) => w.run_range(range),
        None => w.run(),
    }?;

    Ok(())
}
