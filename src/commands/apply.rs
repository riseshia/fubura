use crate::context::Context;
use crate::differ::{build_diff_ops, format_config_diff};
use crate::types::{Config, DiffOp, DiffOpWithTarget, DiffResult};
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, auto_approve: &bool, config: &Config) {
        let mut diff_ops_with_config = vec![];
        let mut diff_result = DiffResult::default();

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

            for diff_op in diff_ops {
                let op_with_config = match diff_op {
                    DiffOp::CreateState => DiffOpWithTarget::CreateState(&ss_config.state),
                    DiffOp::UpdateState => DiffOpWithTarget::UpdateState(&ss_config.state),
                    DiffOp::DeleteState => DiffOpWithTarget::DeleteState(&ss_config.state),
                    DiffOp::AddStateTag => DiffOpWithTarget::AddStateTag(&ss_config.state),
                    DiffOp::RemoteStateTag(tags) => {
                        DiffOpWithTarget::RemoteStateTag(&ss_config.state, tags)
                    }
                    DiffOp::CreateSchedule => {
                        DiffOpWithTarget::CreateSchedule(ss_config.schedule.as_ref().unwrap())
                    }
                    DiffOp::UpdateSchedule => {
                        DiffOpWithTarget::UpdateSchedule(ss_config.schedule.as_ref().unwrap())
                    }
                    DiffOp::DeleteSchedule => {
                        DiffOpWithTarget::DeleteSchedule(ss_config.schedule.as_ref().unwrap())
                    }
                };

                diff_ops_with_config.push(op_with_config);
            }
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

        for diff_op_with_config in diff_ops_with_config.iter() {
            match diff_op_with_config {
                DiffOpWithTarget::CreateState(state) => {
                    println!("Creating state machine: {}", state.name);
                    sfn::create_state_machine(&context.sfn_client, state).await;
                }
                DiffOpWithTarget::UpdateState(state) => {
                    let state_arn = format!("{}{}", state_arn_prefix, state.name);
                    println!("Updating state machine: {}", state.name);
                    sfn::update_state_machine(&context.sfn_client, &state_arn, state).await;
                }
                DiffOpWithTarget::DeleteState(state) => {
                    let state_arn = format!("{}{}", state_arn_prefix, state.name);
                    println!("Deleting state machine: {}", state.name);
                    sfn::delete_state_machine(&context.sfn_client, &state_arn).await;
                }
                DiffOpWithTarget::AddStateTag(state) => {
                    let state_arn = format!("{}{}", state_arn_prefix, state.name);
                    println!("Adding tags to state machine: {}", state.name);
                    sfn::tag_resource(&context.sfn_client, &state_arn, &state.tags).await;
                }
                DiffOpWithTarget::RemoteStateTag(state, removed_keys) => {
                    let state_arn = format!("{}{}", state_arn_prefix, state.name);
                    println!("Removing tags from state machine: {}", state.name);
                    sfn::untag_resource(&context.sfn_client, &state_arn, removed_keys).await;
                }
                DiffOpWithTarget::CreateSchedule(schedule) => {
                    println!("Creating schedule: {}", schedule.name);
                    scheduler::create_schedule(&context.scheduler_client, schedule).await;
                }
                DiffOpWithTarget::UpdateSchedule(schedule) => {
                    println!("Updating schedule: {}", schedule.name);
                    scheduler::update_schedule(&context.scheduler_client, schedule).await;
                }
                DiffOpWithTarget::DeleteSchedule(schedule) => {
                    println!("Deleting schedule: {}", schedule.name);
                    scheduler::delete_schedule(&context.scheduler_client, schedule).await;
                }
            }
        }
    }
}
