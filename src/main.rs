use clap::Parser;

use fubura::cli::{Cli, Commands};
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
