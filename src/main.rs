use clap::Parser;

use fubura::cli::{Cli, Commands, StrKeyVal};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::import::ImportCommand;
use fubura::commands::plan::PlanCommand;
use fubura::context::Context;
use fubura::jsonnet_evaluator;
use fubura::types::SsConfig;

fn load_ss_config(config: &str, ext_str: &[StrKeyVal]) -> SsConfig {
    let config_value = jsonnet_evaluator::eval(config, ext_str).unwrap();

    serde_json::from_value(config_value).unwrap_or_else(|e| {
        panic!("failed to parse config file with error: {}", e);
    })
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply {
            force,
            config,
            ext_str,
        } => {
            let ss_config = load_ss_config(config, ext_str);
            let context = Context::async_default().await;

            ApplyCommand::run(&context, force, &ss_config).await;
        }
        Commands::Plan { config, ext_str } => {
            let ss_config = load_ss_config(config, ext_str);
            let context = Context::async_default().await;

            PlanCommand::run(&context, &ss_config).await;
        }
        Commands::Import {
            config,
            sfn_name,
            schedule_name_with_group,
        } => {
            let context = Context::async_default().await;

            ImportCommand::run(&context, config, sfn_name, schedule_name_with_group).await;
        }
    }
}
