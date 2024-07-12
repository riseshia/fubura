use std::process::exit;

use crate::context::Context;
use crate::differ::print_config_diff;
use crate::types::SsConfig;
use crate::{scheduler, sfn, sts};

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, force: &bool, config: &SsConfig) {
        let sfn_arn_prefix = sts::build_sfn_arn_prefix(context).await;
        let sfn_arn = format!("{}{}", sfn_arn_prefix, config.state.name);

        let remote_sfn = sfn::describe_state_machine(&context.sfn_client, &sfn_arn).await;

        let remote_schedule = if let Some(schedule_config) = &config.schedule {
            scheduler::get_schedule(
                &context.scheduler_client,
                &schedule_config.schedule_name_with_group(),
            )
            .await
        } else {
            None
        };

        print_config_diff(config, &remote_sfn, &remote_schedule);

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
                println!("apply cancelled!");
                exit(1);
            }
        }

        println!("apply called with {:?}!", config)
    }
}
