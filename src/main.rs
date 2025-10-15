use env_logger;
use nntp::NNTPStream;

mod config;
// mod threadpool;
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

    // if app_config.group_lists.is_none() {
    //
    //     let list_options = nntp_stream.list().unwrap();
    //     config::get_group_lists(app_config, list_options)
    //
    //
    //     let answer = MultiSelect::new(
    //         "Group not configured. Select one now:",
    //         nntp_stream.list().unwrap(),
    //     )
    //     .prompt()
    //     .unwrap_or_else(|_| std::process::exit(0));
    //
    //     if answer.len() == 0 {
    //         app_config.group_lists = None
    //     } else {
    //         app_config.group_lists = Some(answer.iter().map(|an| an.name).collect())
    //     }
    // }

    let list_options = nntp_stream.list().unwrap();
    let groups = app_config
        .get_group_lists(list_options.iter().map(move |an| an.clone().name).collect())
        .unwrap();

    println!("made a selection of {} {:#?}", groups.len(), groups);

    // let nntp_clone = nntp_stream.clone()
    worker::Worker::new(&mut nntp_stream, groups).run();

    // let my_stream = smol::stream::iter(groups);
    //
    // let ex = smol::Executor::new();
    //
    // ex.run(async {
    //     // Spawn the set of futures on an executor.
    //     let handles: Vec<smol::Task<()>> = my_stream
    //         .map(|item| {
    //             // Spawn the future on the executor.
    //             ex.spawn(async move {
    //                 //         smol::Timer::after(std::time::Duration::from_secs(5)).await;
    //                 //         format!("result from dynamic future {}", group)
    //                 print!("{}", item)
    //             })
    //         })
    //         .collect()
    //         .await;
    //
    //     // Wait for all of the handles to complete.
    //     for handle in handles {
    //         handle.await;
    //     }
    // })
    // .await;

    // let mut tasks = FuturesUnordered::new();
    // for group in groups {
    //     tasks.push(async move {
    //         smol::Timer::after(std::time::Duration::from_secs(5)).await;
    //         format!("result from dynamic future {}", group)
    //     });
    // }
    //
    // let ex = smol::Executor::new();
    // smol::block_on(futures::try_join!(tasks));

    // tasks.inspect
    // tasks::join();
    // futures::join!(futures)

    // smol::
    // match nntp_stream.group("test") {
    //     Ok(_) => (),
    //     Err(e) => panic!("{}", e),
    // }
    //
    // match nntp_stream.article_by_number(1) {
    //     Ok(Article { headers, body }) => {
    //         for (key, value) in headers.iter() {
    //             println!("{}: {}", key, value)
    //         }
    //         for line in body.iter() {
    //             print!("{}", line)
    //         }
    //     }
    //     Err(e) => panic!("{}", e),
    // js}

    let _ = nntp_stream.quit();
}
