use crate::context::Context;
use crate::types::SsConfig;

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(_context: &Context, config: &SsConfig) {
        // read config
        // fetch resource from aws api
        // diff
        // emit changed resource based on diff
        println!("plan called with {:?}!", config)
    }
}
