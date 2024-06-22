use crate::cli::StrKeyVal;

pub struct PlanCommand;

impl PlanCommand {
    pub fn run(config: &str, ext_str: &Vec<StrKeyVal>) {
        // read config
        // fetch resource from aws api
        // diff
        // emit changed resource based on diff
        println!("plan called with {}, {:?}!", config, ext_str)
    }
}
