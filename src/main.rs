use std::path::Path;

use clap::Parser;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::import::ImportCommand;
use fubura::commands::plan::PlanCommand;
use fubura::context::FuburaContext;
use fubura::types::Config;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Apply {
            auto_approve,
            config_path,
            ext_str,
            target,
            json_diff_path,
        } => {
            let config = Config::load_from_path(config_path, ext_str);
            let mut context = FuburaContext::async_default().await;
            context.targets.clone_from(target);
            context.json_diff_path.clone_from(json_diff_path);

            ApplyCommand::run(&context, auto_approve, &config).await
        }
        Commands::Plan {
            config_path,
            ext_str,
            target,
            json_diff_path,
        } => {
            let config = Config::load_from_path(config_path, ext_str);
            let mut context = FuburaContext::async_default().await;
            context.targets.clone_from(target);
            context.json_diff_path.clone_from(json_diff_path);

            PlanCommand::run(&context, &config).await
        }
        Commands::Import {
            config_path,
            ext_str,
            sfn_name,
            schedule_name_with_group,
        } => {
            let config_exist = Path::new(config_path).exists();
            let config = if config_exist {
                Config::load_from_path(config_path, ext_str)
            } else {
                Config::default()
            };

            let context = FuburaContext::async_default().await;

            ImportCommand::run(
                &context,
                config_path,
                config,
                sfn_name,
                schedule_name_with_group,
            )
            .await
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
