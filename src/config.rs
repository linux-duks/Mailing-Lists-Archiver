use crate::range_inputs;
use clap::{Args, Parser, ValueHint};
use config::Config;
use glob::glob;
use inquire::MultiSelect;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

// TODO: test use confique::Config;

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
    pub hostname: Option<String>,
    #[arg(short, long, default_value = "119")]
    pub port: u16,
    #[arg(short, long, default_value = "./output", value_hint = ValueHint::DirPath)]
    pub output_dir: String,
    #[arg(short, long, default_value = "1")]
    pub nthreads: u8,

    #[arg(long)]
    group_lists: Option<Vec<String>>,
    /// comma separated values, or dash separated ranges, like low-high
    #[arg(long)]
    article_range: Option<String>,
}

pub fn read_config() -> Result<AppConfig, anyhow::Error> {
    let opts = Opts::parse();

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
            glob(&opts.config_file)?
                .map(|path| config::File::from(path.unwrap()))
                .collect::<Vec<_>>(),
        );

    let config = config.build().unwrap();

    let app_config: AppConfig = config.try_deserialize()?;

    Ok(app_config)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    // Io(#[from] io::Error),
    #[error("invalid list selection. At least one should be configured, or selected in runtime")]
    ListSelectionEmpty,
    #[error("configured list(s) not available in server. {} Lists with error: {}", unavailable_lists.len(), unavailable_lists.iter().map(|x| x.to_string() + ",").collect::<String>()
)]
    ConfiguredListsNotAvailable { unavailable_lists: Vec<String> },
    #[error("none of the configured lists are available in server")]
    AllListsUnavailable,
}

impl AppConfig {
    /// returns the lists ready to use
    ///
    /// Takes lists from config. If none configured, prompt user for selection.
    /// If list was configured, check if selected lists are available in the server
    /// Return only available lists
    pub fn get_group_lists(
        &mut self,
        list_options: Vec<String>,
    ) -> Result<Vec<String>, ConfigError> {
        if self.group_lists.is_none() {
            log::info!("No group_lists defined");
            let answer = MultiSelect::new("No groups selected. Select them now:", list_options)
                .prompt()
                .unwrap_or_else(|_| std::process::exit(0));

            if answer.is_empty() {
                log::info!("empty selection");
                self.group_lists = None;
                Err(ConfigError::ListSelectionEmpty)
            } else {
                // save selection to a file
                // TODO: move fo file_utils
                let mut selected_lists = HashMap::new();
                selected_lists.insert("group_lists", answer.clone());

                let f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open("nntp_config_selected_lists.yml")
                    .expect("Couldn't open file");

                serde_yaml::to_writer(f, &selected_lists).unwrap();

                Ok(answer)
            }
        } else {
            let mut group_lists = self.group_lists.clone().unwrap();
            group_lists.dedup();
            let item_set: HashSet<_> = list_options.iter().collect();
            group_lists.retain(|item| item_set.contains(item));
            let (valid, invalid): (Vec<_>, Vec<_>) = group_lists
                .into_iter()
                .partition(|item| item_set.contains(item));

            if valid.is_empty() {
                return Err(ConfigError::AllListsUnavailable);
            }
            if !invalid.is_empty() {
                log::warn!(
                    "Some lists are unavailable: {}",
                    ConfigError::ConfiguredListsNotAvailable {
                        unavailable_lists: invalid
                    }
                );
            }

            Ok(valid)
        }
    }

    pub fn get_article_range(&self) -> Option<impl Iterator<Item = usize>> {
        match &self.article_range {
            Some(range_text) => {
                // range and multiple lists
                if self.group_lists.as_ref().is_some_and(|x| x.len() > 1) {
                    log::warn!(
                        "article_range used with group_lists with more than one list. This is likely an error"
                    );
                }
                return match range_inputs::parse_sequence(range_text) {
                    Ok(range) => Some(range),
                    Err(e) => {
                        log::error!("Invalid article_range input: {e}");
                        None
                    }
                };
            }
            None => {
                return None;
            }
        }
    }
}
