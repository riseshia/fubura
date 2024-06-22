use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = false)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

use fubura::commands::apply::ApplyCommand;
use fubura::commands::plan::PlanCommand;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply { force, config, ext_str } => {
            println!("{:?}", ext_str);
            ApplyCommand::run(force, config);
        }
        Commands::Plan { config, ext_str } => {
            println!("{:?}", ext_str);
            PlanCommand::run(config);
        }
    }
}
