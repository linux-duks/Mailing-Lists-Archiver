#![allow(clippy::needless_return)]

use env_logger::Env;

use mlh_archiver::Result;
use mlh_archiver::config;
use mlh_archiver::start;

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let mut app_config = config::read_config().unwrap();
    return start(&mut app_config);
}
