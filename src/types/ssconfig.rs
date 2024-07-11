use serde::{Deserialize, Serialize};

use super::{Schedule, StateMachine};

#[derive(Deserialize, Serialize, Debug)]
pub struct SsConfig {
    pub state: StateMachine,
    pub schedule: Option<Schedule>,
}
