use std::process::exit;

use crate::context::Context;
use crate::types::SsConfig;

use super::plan::PlanCommand;

pub struct ApplyCommand;

impl ApplyCommand {
    pub async fn run(context: &Context, force: &bool, config: &SsConfig) {
        if !force {
            PlanCommand::run(context, config).await;

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
                exit(1)
            }
        }

        println!("apply called with {:?}!", config)
    }
}
