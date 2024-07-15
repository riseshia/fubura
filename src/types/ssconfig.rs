use serde::{Deserialize, Serialize};

use super::{Schedule, StateMachine};

fn default_delete_flag() -> bool {
    false
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SsConfig {
    pub state: StateMachine,
    pub schedule: Option<Schedule>,
    #[serde(default = "default_delete_flag")]
    pub delete_all: bool,
    #[serde(default = "default_delete_flag")]
    pub delete_schedule: bool,
}
