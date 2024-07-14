use crate::context::Context;
use crate::differ::{build_diff_ops, print_config_diff, print_diff_ops};
use crate::types::SsConfig;
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, force: &bool, config: &SsConfig) {
        let state_arn_prefix = sts::build_state_arn_prefix(context).await;
        let state_arn = format!("{}{}", state_arn_prefix, config.state.name);

        let remote_state =
            sfn::describe_state_machine_with_tags(&context.sfn_client, &state_arn).await;

        let remote_schedule = if let Some(schedule_config) = &config.schedule {
            scheduler::get_schedule(
                &context.scheduler_client,
                &schedule_config.schedule_name_with_group(),
            )
            .await
        } else {
            None
        };

        print_config_diff(config, &remote_state, &remote_schedule);
        let diff_ops = build_diff_ops(config, &remote_state, &remote_schedule);
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

        println!("apply called with {:?}!", config)
    }
}
