use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = false)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// apply config
    Apply {
        /// Skip confirm changes, and apply it immediately.
        #[clap(long = "auto-approve", short = 'a')]
        auto_approve: bool,
        /// Config file path
        #[clap(long = "config", short = 'c', default_value = "fubura.jsonnet")]
        config_path: String,
        /// filter with target state machine names
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
        /// filter with target state machine names
        #[clap(long = "target", short = 't', value_name = "key=[val]")]
        target: Option<Vec<String>>,
        /// Specify path to diff result as json
        #[clap(long = "diff-as-json", short = 'o', value_name = "output path")]
        json_diff_path: Option<String>,
        /// Emit logs for debugging
        #[clap(long = "debug")]
        debug_mode: bool,
    },
    /// plan config
    Plan {
        /// Config file path
        #[clap(long = "config", short = 'c', default_value = "fubura.jsonnet")]
        config_path: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        /// jsonnet --ext-str options
        ext_str: Vec<StrKeyVal>,
        /// filter with target state machine names
        #[clap(long = "target", short = 't', value_name = "key=[val]")]
        target: Option<Vec<String>>,
        /// Specify path to diff result as json
        #[clap(long = "diff-as-json", short = 'o', value_name = "output path")]
        json_diff_path: Option<String>,
        /// Emit logs for debugging
        #[clap(long = "debug")]
        debug_mode: bool,
    },
    /// import state machine to specified config file
    Import {
        /// Where to import its config
        #[clap(long = "config", short = 'c', default_value = "fubura.jsonnet")]
        config_path: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
        /// import target state machine arn
        #[arg(long = "sfn-name", short = 'f', value_name = "state-machine-name")]
        sfn_name: String,
        /// import target scheduler name with group (optional)
        #[arg(
            long = "scheduler-name-with-group",
            short = 's',
            value_name = "group-name/schedule-name"
        )]
        schedule_name_with_group: Option<String>,
        /// Emit logs for debugging
        #[clap(long = "debug")]
        debug_mode: bool,
    },
}

#[derive(Clone, Debug)]
pub struct StrKeyVal {
    pub var: String,
    pub val: Option<String>,
}

impl From<&str> for StrKeyVal {
    fn from(s: &str) -> Self {
        if let Some((key, val)) = s.split_once('=') {
            Self {
                var: key.into(),
                val: Some(val.into()),
            }
        } else {
            Self {
                var: s.into(),
                val: None,
            }
        }
    }
}
