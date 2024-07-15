use super::{Schedule, StateMachine};

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum DiffOp {
    CreateState,
    UpdateState,
    AddStateTag,
    RemoteStateTag(Vec<String>),
    CreateSchedule,
    UpdateSchedule,
    DeleteSchedule,
    DeleteState,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DiffOpWithTarget<'a> {
    CreateState(&'a StateMachine),
    UpdateState(&'a StateMachine),
    AddStateTag(&'a StateMachine),
    RemoteStateTag(&'a StateMachine, Vec<String>),
    CreateSchedule(&'a Schedule),
    UpdateSchedule(&'a Schedule),
    DeleteSchedule(&'a Schedule),
    DeleteState(&'a StateMachine),
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_diff_op_ordering_delete_state_after_delete_schedule() {
        let mut actual_diff_ops = vec![
            DiffOp::CreateState,
            DiffOp::UpdateState,
            DiffOp::DeleteState,
            DiffOp::AddStateTag,
            DiffOp::RemoteStateTag(vec!["tag".to_string()]),
            DiffOp::CreateSchedule,
            DiffOp::UpdateSchedule,
            DiffOp::DeleteSchedule,
        ];

        actual_diff_ops.sort();

        let expected = vec![
            DiffOp::CreateState,
            DiffOp::UpdateState,
            DiffOp::AddStateTag,
            DiffOp::RemoteStateTag(vec!["tag".to_string()]),
            DiffOp::CreateSchedule,
            DiffOp::UpdateSchedule,
            DiffOp::DeleteSchedule,
            DiffOp::DeleteState,
        ];

        similar_asserts::assert_eq!(expected, actual_diff_ops);
    }
}
