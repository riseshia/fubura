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
        #[arg(short, long, default_value = "exported-sfn-confg.jsonnet")]
        config: String,
        /// export target state machine arn
        #[arg(long = "sfn-arn")]
        sfn_arn: String,
        /// export target scheduler arn (optional)
        #[arg(long = "schedule-arn")]
        schedule_arn: Option<String>,
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
