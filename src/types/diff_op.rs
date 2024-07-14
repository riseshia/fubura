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

#[derive(Debug, PartialEq, Clone)]
pub enum DiffOpWithTarget<'a> {
    CreateSfn(&'a StateMachine),
    UpdateSfn(&'a StateMachine),
    AddSfnTag(&'a StateMachine),
    RemoveSfnTag(&'a StateMachine, Vec<String>),
    DeleteSfn(&'a StateMachine),
    CreateSchedule(&'a Schedule),
    UpdateSchedule(&'a Schedule),
    DeleteSchedule(&'a Schedule),
}
