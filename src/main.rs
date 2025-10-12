use clap::{Args, Parser, ValueHint};

use config::Config;
// TODO: test use confique::Config;
use glob::glob;

use inquire::Select;

use nntp::{Article, NNTPStream};

#[derive(Debug, Parser, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Opts {
    // config file location override
    #[arg(short, long, default_value = "nntp_config*", value_hint = ValueHint::FilePath)]
    config_file: String,

    #[clap(flatten)]
    app_config: Option<AppConfig>,
}

#[derive(Debug, Args, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
pub struct AppConfig {
    #[arg(short = 'H', long)]
    hostname: Option<String>,
    #[arg(short, long, default_value = "119")]
    port: u16,

    #[arg(long)]
    list_group: Option<String>,
}

fn main() {
    let opts = Opts::parse();

    // println!("{:#?}", opts);

    let base_config = match opts.app_config {
        Some(app_config) => app_config,
        None => AppConfig::default(),
    };

    let defaults = Config::try_from(&base_config).unwrap();

    // TODO: config layering is not working properly
    let config = Config::builder()
        .set_default("port", 119)
        .unwrap()
        .add_source(defaults)
        // env variable config
        .add_source(
            config::Environment::with_prefix("NNTP")
                .try_parsing(true)
                .separator("_"),
        )
        // TODO:  add xdg_home config
        .add_source(
            glob(&opts.config_file)
                .unwrap()
                .map(|path| config::File::from(path.unwrap()))
                .collect::<Vec<_>>(),
        );

    let config = config.build().unwrap();

    let mut app_config: AppConfig = config.try_deserialize().unwrap();

    let mut nntp_stream = match NNTPStream::connect((app_config.hostname.unwrap(), app_config.port))
    {
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

    if app_config.list_group.is_none() {
        let answer = Select::new(
            "Group not configured. Select one now:",
            nntp_stream.list().unwrap(),
        )
        .prompt()
        .unwrap_or_else(|_| std::process::exit(0));

        app_config.list_group = Some(answer.name)
    }

    match nntp_stream.group(&app_config.list_group.unwrap()) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }

    match nntp_stream.article_by_number(1) {
        Ok(Article { headers, body }) => {
            for (key, value) in headers.iter() {
                println!("{}: {}", key, value)
            }
            for line in body.iter() {
                print!("{}", line)
            }
        }
        Err(e) => panic!("{}", e),
    }

    let _ = nntp_stream.quit();
}
