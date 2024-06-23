use clap::Parser;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::plan::PlanCommand;
use fubura::commands::export::ExportCommand;
use fubura::jsonnet_evaluator;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply { force, config, ext_str } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();

            ApplyCommand::run(force, &config).await;
        }
        Commands::Plan { config, ext_str } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();

            PlanCommand::run(&config).await;
        }
        Commands::Export { config, sfn_arn, schedule_arn } => {
            ExportCommand::run(config, sfn_arn, schedule_arn).await;
        }
    }
}
