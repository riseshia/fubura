use std::process::exit;

use crate::cli::StrKeyVal;

use super::plan::PlanCommand;

pub struct ApplyCommand;

impl ApplyCommand {
    pub fn run(force: &bool, config: &str, ext_str: &Vec<StrKeyVal>) {
        if !force {
            PlanCommand::run(config, ext_str);

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

        println!("apply called with {}, {:?}!", config, ext_str)
    }
}
