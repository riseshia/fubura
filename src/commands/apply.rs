use std::collections::HashMap;

use anyhow::{bail, Result};
use tracing::info;

use crate::context::FuburaContext;
use crate::differ::diff;
use crate::types::{Config, DiffOp, DiffResult, SsConfig};
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &FuburaContext, auto_approve: &bool, config: &Config) -> Result<()> {
        let ss_config_by_name: HashMap<String, &SsConfig> = HashMap::from_iter(
            config
                .ss_configs
                .iter()
                .map(|ss_config| (ss_config.state.name.clone(), ss_config)),
        );

        let diff_result = diff(context, config).await?;

        if let Some(json_diff_path) = &context.json_diff_path {
            write_result_to_path(json_diff_path, &diff_result)?;
        }

        if diff_result.no_change {
            return Ok(());
        }

        if !auto_approve {
            print!(
                r#"
Do you want apply this change?
Only 'yes' will be accepted to approve.

Enter a value: "#
            );
            use text_io::read;
            let response: String = read!("{}\n");

            if response != "yes" {
                bail!("apply cancelled!");
            }
        }

        let state_arn_prefix = sts::build_state_arn_prefix(context).await;
        for diff_ops_for_ss in diff_result.detail_diff_ops.iter() {
            let ss_config = *ss_config_by_name.get(&diff_ops_for_ss.state_name).unwrap();
            let state = &ss_config.state;

            for diff_op in diff_ops_for_ss.diff_ops.iter() {
                match diff_op {
                    DiffOp::CreateState => {
                        info!("Creating state machine: {}", state.name);
                        sfn::create_state_machine(&context.sfn_client, state).await?;
                    }
                    DiffOp::UpdateState => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        info!("Updating state machine: {}", state.name);
                        sfn::update_state_machine(&context.sfn_client, &state_arn, state).await?;
                    }
                    DiffOp::DeleteState => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        info!("Deleting state machine: {}", state.name);
                        sfn::delete_state_machine(&context.sfn_client, &state_arn).await?;
                    }
                    DiffOp::AddStateTag => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        info!("Adding tags to state machine: {}", state.name);
                        sfn::tag_resource(&context.sfn_client, &state_arn, &state.tags).await?;
                    }
                    DiffOp::RemoveStateTag(removed_keys) => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        info!("Removing tags from state machine: {}", state.name);
                        sfn::untag_resource(&context.sfn_client, &state_arn, removed_keys).await?;
                    }
                    DiffOp::CreateSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        info!("Creating schedule: {}", schedule.name);
                        scheduler::create_schedule(&context.scheduler_client, schedule).await?;
                    }
                    DiffOp::UpdateSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        info!("Updating schedule: {}", schedule.name);
                        scheduler::update_schedule(&context.scheduler_client, schedule).await?;
                    }
                    DiffOp::DeleteSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        info!("Deleting schedule: {}", schedule.name);
                        scheduler::delete_schedule(&context.scheduler_client, schedule).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn write_result_to_path(output_path: &str, diff_result: &DiffResult) -> Result<()> {
    let json_diff = serde_json::to_string_pretty(diff_result).unwrap();
    std::fs::write(output_path, json_diff)?;
    Ok(())
}
