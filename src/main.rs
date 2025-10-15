use nntp::NNTPStream;

mod config;
mod worker;

fn main() {
    env_logger::init();

    let mut app_config = config::read_config().unwrap();

    let mut nntp_stream =
        match NNTPStream::connect((app_config.hostname.clone().unwrap(), app_config.port)) {
            Ok(stream) => stream,
            Err(e) => panic!("{}", e),
        };

    match nntp_stream.capabilities() {
        Ok(lines) => {
            for line in lines.iter() {
                print!("{}", line);
            }
        }
        Err(e) => panic!("{}", e),
    }

    let list_options = nntp_stream.list().unwrap();
    let groups = app_config
        .get_group_lists(list_options.iter().map(move |an| an.clone().name).collect())
        .unwrap();

    println!("made a selection of {} {:#?}", groups.len(), groups);

    worker::Worker::new(&mut nntp_stream, groups).run();

    let _ = nntp_stream.quit();
}
