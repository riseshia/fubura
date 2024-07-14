use crate::context::Context;
use crate::differ::{build_diff_ops, print_config_diff, print_diff_ops};
use crate::types::{Config, DiffOp};
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, force: &bool, config: &Config) {
        let ss_config = config.ss_configs.first().unwrap();

        let state_arn_prefix = sts::build_state_arn_prefix(context).await;
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

        print_config_diff(ss_config, &remote_state, &remote_schedule);
        let diff_ops = build_diff_ops(ss_config, &remote_state, &remote_schedule);
        print_diff_ops(&diff_ops);

        if !force {
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

        for diff_op in diff_ops {
            match diff_op {
                DiffOp::CreateSfn => {
                    println!("Creating state machine: {}", ss_config.state.name);
                    sfn::create_state_machine(&context.sfn_client, &ss_config.state).await;
                }
                DiffOp::UpdateSfn => {
                    println!("Updating state machine: {}", ss_config.state.name);
                    sfn::update_state_machine(&context.sfn_client, &state_arn, &ss_config.state)
                        .await;
                }
                DiffOp::DeleteSfn => {
                    println!("Deleting state machine: {}", ss_config.state.name);
                    sfn::delete_state_machine(&context.sfn_client, &state_arn).await;
                }
                DiffOp::AddSfnTag => {
                    println!("Adding tags to state machine: {}", ss_config.state.name);
                    sfn::tag_resource(&context.sfn_client, &state_arn, &ss_config.state.tags).await;
                }
                DiffOp::RemoveSfnTag(_) => {
                    println!("Removing tags from state machine: {}", ss_config.state.name);
                    sfn::untag_resource(&context.sfn_client, &state_arn, &ss_config.state.tags)
                        .await;
                }
                DiffOp::CreateSchedule => {
                    println!(
                        "Creating schedule: {}",
                        ss_config.schedule.as_ref().unwrap().name
                    );
                    scheduler::create_schedule(
                        &context.scheduler_client,
                        ss_config.schedule.as_ref().unwrap(),
                    )
                    .await;
                }
                DiffOp::UpdateSchedule => {
                    println!(
                        "Updating schedule: {}",
                        ss_config.schedule.as_ref().unwrap().name
                    );
                    scheduler::update_schedule(
                        &context.scheduler_client,
                        ss_config.schedule.as_ref().unwrap(),
                    )
                    .await;
                }
                DiffOp::DeleteSchedule => {
                    println!(
                        "Deleting schedule: {}",
                        ss_config.schedule.as_ref().unwrap().name
                    );
                    scheduler::delete_schedule(
                        &context.scheduler_client,
                        ss_config.schedule.as_ref().unwrap(),
                    )
                    .await;
                }
            }
        }
    }
}
