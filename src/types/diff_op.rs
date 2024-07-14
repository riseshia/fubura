use super::{Schedule, StateMachine};

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum DiffOp {
    CreateState,
    UpdateState,
    DeleteState,
    AddStateTag,
    RemoteStateTag(Vec<String>),
    CreateSchedule,
    UpdateSchedule,
    DeleteSchedule,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DiffOpWithTarget<'a> {
    CreateState(&'a StateMachine),
    UpdateState(&'a StateMachine),
    DeleteState(&'a StateMachine),
    AddStateTag(&'a StateMachine),
    RemoteStateTag(&'a StateMachine, Vec<String>),
    CreateSchedule(&'a Schedule),
    UpdateSchedule(&'a Schedule),
    DeleteSchedule(&'a Schedule),
}
