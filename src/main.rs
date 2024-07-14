use std::path::Path;

use clap::Parser;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::import::ImportCommand;
use fubura::commands::plan::PlanCommand;
use fubura::context::Context;
use fubura::types::Config;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply {
            force,
            config,
            ext_str,
            target,
        } => {
            let config = Config::load_from_path(config, ext_str);
            let mut context = Context::async_default().await;
            context.targets.clone_from(target);

            ApplyCommand::run(&context, force, &config).await;
        }
        Commands::Plan {
            config,
            ext_str,
            target,
        } => {
            let config = Config::load_from_path(config, ext_str);
            let mut context = Context::async_default().await;
            context.targets.clone_from(target);

            PlanCommand::run(&context, &config).await;
        }
        Commands::Import {
            config,
            ext_str,
            sfn_name,
            schedule_name_with_group,
        } => {
            let config_path = config;
            let config_exist = Path::new(config).exists();
            let config = if config_exist {
                Config::load_from_path(config, ext_str)
            } else {
                Config::default()
            };

            let context = Context::async_default().await;

            ImportCommand::run(
                &context,
                config_path,
                config,
                sfn_name,
                schedule_name_with_group,
            )
            .await;
        }
    }
}
