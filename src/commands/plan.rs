use serde_json::Value;

pub struct PlanCommand;

impl PlanCommand {
    pub fn run(config: &Value) {
        // read config
        // fetch resource from aws api
        // diff
        // emit changed resource based on diff
        println!("plan called with {:?}!", config)
    }
}
