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
        force: bool
    },
    /// plan config
    Plan,
}

use fubura::commands::plan::PlanCommand;
use fubura::commands::apply::ApplyCommand;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Apply { force }=> {
            ApplyCommand::run(force);
        }
        Commands::Plan => {
            PlanCommand::run();
        }
    }
}
