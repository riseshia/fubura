use serde_json::Value;

use crate::context::Context;

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(_context: &Context, config: &Value) {
        // read config
        // fetch resource from aws api
        // diff
        // emit changed resource based on diff
        println!("plan called with {:?}!", config)
    }
}
