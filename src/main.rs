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
    /// apply schedules to EventBridge Scheduler
    Apply {
        /// Skip to check changes, but only apply it.
        #[arg(short, long)]
        force: bool
    },
    /// plan schedules from EventBridge Scheduler
    Plan,
    /// generate schedules bootstrap
    Init,
}

use eber::commands::init::InitCommand;
use eber::commands::plan::PlanCommand;
use eber::commands::apply::ApplyCommand;

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Apply { force }=> {
            ApplyCommand::run(force);
        }
        Commands::Plan => {
            PlanCommand::run();
        }
        Commands::Init => {
            InitCommand::run();
        }
    }
}
