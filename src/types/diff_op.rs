use serde::{Serialize, Serializer};

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

impl DiffOp {
    pub fn op_for_report(op: &DiffOp) -> &DiffOp {
        match op {
            DiffOp::AddStateTag => &DiffOp::UpdateState,
            DiffOp::RemoteStateTag(_) => &DiffOp::UpdateState,
            op => op,
        }
    }

    pub fn op_type(&self) -> &str {
        match self {
            DiffOp::CreateState => "create_state",
            DiffOp::UpdateState => "update_state",
            DiffOp::AddStateTag => "add_state_tag",
            DiffOp::RemoteStateTag(_) => "remote_state_tag",
            DiffOp::CreateSchedule => "create_schedule",
            DiffOp::UpdateSchedule => "update_schedule",
            DiffOp::DeleteSchedule => "delete_schedule",
            DiffOp::DeleteState => "delete_state",
        }
    }
}

impl Serialize for DiffOp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.op_type())
    }
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
