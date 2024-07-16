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
        /// Config file
        #[arg(short, long, default_value = "fubura.jsonnet")]
        config: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
        #[clap(long = "target", short = 't', value_name = "key=[val]")]
        target: Option<Vec<String>>,
    },
    /// plan config
    Plan {
        /// Config file
        #[arg(short, long, default_value = "fubura.jsonnet")]
        config: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
        #[clap(long = "target", short = 't', value_name = "key=[val]")]
        target: Option<Vec<String>>,
    },
    /// import state machine to specified config file
    Import {
        /// Where to import its config
        #[arg(
            short,
            long,
            default_value = "fubura.jsonnet",
            value_name = "import-path"
        )]
        config: String,
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
