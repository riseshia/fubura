use super::{Schedule, StateMachine};

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum DiffOp {
    CreateSfn,
    CreateSchedule,
    UpdateSfn,
    UpdateSchedule,
    AddSfnTag,
    RemoveSfnTag(Vec<String>),
    DeleteSfn,
    DeleteSchedule,
}
