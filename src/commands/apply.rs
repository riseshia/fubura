pub struct ApplyCommand;

use std::process::exit;

use super::plan::PlanCommand;

impl ApplyCommand {
    pub fn run(force: &bool) {
        if !force {
            PlanCommand::run();

            print!(
                r#"
Do you want apply this change?
Only 'yes' will be accepted to approve.

Enter a value: "#
            );
            use text_io::read;
            let response: String = read!("{}\n");

            if response != "yes" {
                println!("apply canceled!");
                exit(1)
            }
        }

        println!("apply called!")
    }
}
