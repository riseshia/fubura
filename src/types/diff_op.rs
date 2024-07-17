use serde::{Serialize, Serializer};

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

impl Serialize for DiffOp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DiffOp::CreateState => serializer.serialize_str("create_state"),
            DiffOp::UpdateState => serializer.serialize_str("update_state"),
            DiffOp::AddStateTag => serializer.serialize_str("add_state_tag"),
            DiffOp::RemoteStateTag(_) => serializer.serialize_str("remote_state_tag"),
            DiffOp::CreateSchedule => serializer.serialize_str("create_schedule"),
            DiffOp::UpdateSchedule => serializer.serialize_str("update_schedule"),
            DiffOp::DeleteSchedule => serializer.serialize_str("delete_schedule"),
            DiffOp::DeleteState => serializer.serialize_str("delete_state"),
        }
    }
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

    #[tokio::test]
    async fn test_diff_op_serialize() {
        let actual_diff_ops = vec![
            DiffOp::CreateState,
            DiffOp::UpdateState,
            DiffOp::DeleteState,
            DiffOp::AddStateTag,
            DiffOp::RemoteStateTag(vec!["tag".to_string()]),
            DiffOp::CreateSchedule,
            DiffOp::UpdateSchedule,
            DiffOp::DeleteSchedule,
        ];

        let serialized = serde_json::to_string_pretty(&actual_diff_ops).unwrap();
        let actual = serde_json::from_str::<Vec<String>>(&serialized).unwrap();

        let expected = vec![
            "create_state",
            "update_state",
            "delete_state",
            "add_state_tag",
            "remote_state_tag",
            "create_schedule",
            "update_schedule",
            "delete_schedule",
        ];

        similar_asserts::assert_eq!(expected, actual);
    }
}
