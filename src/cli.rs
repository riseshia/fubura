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
        /// Skip to check changes, but only apply it.
        #[arg(short, long)]
        force: bool,
        /// Config file
        #[arg(short, long, default_value = "fubura.jsonnet")]
        config: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
    },
    /// plan config
    Plan {
        /// Config file
        #[arg(short, long, default_value = "fubura.jsonnet")]
        config: String,
        #[clap(long = "ext-str", short = 'V', value_name = "key=[val]")]
        ext_str: Vec<StrKeyVal>,
    },
    /// export state machine config
    Export {
        /// Where to export its config
        #[arg(
            short,
            long,
            default_value = "exported-sfn-confg.jsonnet",
            value_name = "export-path"
        )]
        config: String,
        /// export target state machine arn
        #[arg(long = "sfn-arn", short = 'f', value_name = "state-machine-arn")]
        sfn_arn: String,
        /// export target scheduler name with group (optional)
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
