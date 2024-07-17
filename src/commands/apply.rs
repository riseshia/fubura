use std::collections::HashMap;

use crate::context::Context;
use crate::differ::{build_diff_ops, format_config_diff};
use crate::types::{Config, DiffOp, DiffResult, SsConfig};
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, auto_approve: &bool, config: &Config) -> DiffResult {
        let mut diff_result = DiffResult::default();
        let ss_config_by_name: HashMap<String, &SsConfig> = HashMap::from_iter(
            config
                .ss_configs
                .iter()
                .map(|ss_config| (ss_config.state.name.clone(), ss_config)),
        );

        let state_arn_prefix = sts::build_state_arn_prefix(context).await;

        let target_ss_configs = config.target_ss_configs(&context.targets);

        for ss_config in target_ss_configs {
            let state_arn = format!("{}{}", state_arn_prefix, ss_config.state.name);

            let remote_state =
                sfn::describe_state_machine_with_tags(&context.sfn_client, &state_arn).await;

            let remote_schedule = if let Some(schedule_config) = &ss_config.schedule {
                scheduler::get_schedule(
                    &context.scheduler_client,
                    &schedule_config.schedule_name_with_group(),
                )
                .await
            } else {
                None
            };

            let diff_ops = build_diff_ops(ss_config, &remote_state, &remote_schedule);
            for diff_op in diff_ops.iter() {
                diff_result.append_diff_op(&ss_config.state.name, diff_op)
            }

            let text_diff =
                format_config_diff(ss_config, &remote_state, &remote_schedule, &diff_ops);
            println!("{}", &text_diff);
            diff_result.append_text_diff(text_diff);
        }

        println!("\nFubura will:");
        for (op, count) in diff_result.summary.iter() {
            println!("    {}: {}", op, count);
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
                panic!("apply cancelled!");
            }
        }

        for diff_ops_for_ss in diff_result.detail_diff_ops.iter() {
            let ss_config = *ss_config_by_name.get(&diff_ops_for_ss.state_name).unwrap();
            let state = &ss_config.state;

            for diff_op in diff_ops_for_ss.diff_ops.iter() {
                match diff_op {
                    DiffOp::CreateState => {
                        println!("Creating state machine: {}", state.name);
                        sfn::create_state_machine(&context.sfn_client, state).await;
                    }
                    DiffOp::UpdateState => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        println!("Updating state machine: {}", state.name);
                        sfn::update_state_machine(&context.sfn_client, &state_arn, state).await;
                    }
                    DiffOp::DeleteState => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        println!("Deleting state machine: {}", state.name);
                        sfn::delete_state_machine(&context.sfn_client, &state_arn).await;
                    }
                    DiffOp::AddStateTag => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        println!("Adding tags to state machine: {}", state.name);
                        sfn::tag_resource(&context.sfn_client, &state_arn, &state.tags).await;
                    }
                    DiffOp::RemoteStateTag(removed_keys) => {
                        let state_arn = format!("{}{}", state_arn_prefix, state.name);
                        println!("Removing tags from state machine: {}", state.name);
                        sfn::untag_resource(&context.sfn_client, &state_arn, removed_keys).await;
                    }
                    DiffOp::CreateSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        println!("Creating schedule: {}", schedule.name);
                        scheduler::create_schedule(&context.scheduler_client, schedule).await;
                    }
                    DiffOp::UpdateSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        println!("Updating schedule: {}", schedule.name);
                        scheduler::update_schedule(&context.scheduler_client, schedule).await;
                    }
                    DiffOp::DeleteSchedule => {
                        let schedule = ss_config.schedule.as_ref().unwrap();
                        println!("Deleting schedule: {}", schedule.name);
                        scheduler::delete_schedule(&context.scheduler_client, schedule).await;
                    }
                }
            }
        }

        diff_result
    }
}
