use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
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
    },
    /// plan config
    Plan {
        /// Config file
        #[arg(short, long, default_value = "fubura.jsonnet")]
        config: String,
    },
}

use fubura::commands::apply::ApplyCommand;
use fubura::commands::plan::PlanCommand;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Apply { force, config } => {
            ApplyCommand::run(force, config);
        }
        Commands::Plan { config } => {
            PlanCommand::run(config);
        }
    }
}
