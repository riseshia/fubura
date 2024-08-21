use std::path::Path;

use clap::Parser;

use tracing::info;
use tracing_subscriber::prelude::*;

use fubura::cli::{Cli, Commands};
use fubura::commands::apply::ApplyCommand;
use fubura::commands::import::ImportCommand;
use fubura::commands::plan::PlanCommand;
use fubura::context::FuburaContext;
use fubura::fast_exit;
use fubura::types::Config;

fn set_log_level(debug_mode: &bool) {
    let fubura_level = if *debug_mode {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    // Do not log info level for dependencies which is too verbose
    let dependency_level = if *debug_mode {
        tracing::Level::DEBUG
    } else {
        tracing::Level::ERROR
    };

    let format = tracing_subscriber::fmt::format()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .compact();

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("fubura", fubura_level)
        .with_default(dependency_level);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().event_format(format))
        .with(filter)
        .init();

    info!("Set log level: {:?}", fubura_level);
}

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
            debug_mode,
        } => {
            set_log_level(debug_mode);

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
            debug_mode,
        } => {
            set_log_level(debug_mode);

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
            debug_mode,
        } => {
            set_log_level(debug_mode);

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
        fast_exit!("{}", e);
    }
}
