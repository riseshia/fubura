use clap::Parser;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::plan::PlanCommand;
use fubura::jsonnet_evaluator;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply { force, config, ext_str } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();

            ApplyCommand::run(force, &config);
        }
        Commands::Plan { config, ext_str } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();

            PlanCommand::run(&config);
        }
    }
}
