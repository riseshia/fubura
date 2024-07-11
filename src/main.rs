use clap::Parser;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::export::ExportCommand;
use fubura::commands::plan::PlanCommand;
use fubura::context::Context;
use fubura::jsonnet_evaluator;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply {
            force,
            config,
            ext_str,
        } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();
            let context = Context::async_default().await;

            ApplyCommand::run(&context, force, &config).await;
        }
        Commands::Plan { config, ext_str } => {
            let config = jsonnet_evaluator::eval(config, ext_str).unwrap();
            let context = Context::async_default().await;

            PlanCommand::run(&context, &config).await;
        }
        Commands::Export {
            config,
            sfn_name,
            schedule_name_with_group,
        } => {
            let context = Context::async_default().await;

            ExportCommand::run(&context, config, sfn_name, schedule_name_with_group).await;
        }
    }
}
