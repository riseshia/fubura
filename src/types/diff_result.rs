use std::collections::HashMap;

use console::strip_ansi_codes;
use serde::Serialize;

use super::DiffOp;

type OpName = String;

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DiffOpsForSs {
    pub state_name: String,
    pub diff_ops: Vec<DiffOp>,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DiffResult {
    pub text_diff: Vec<String>,
    // diff_ops list used for report to user. tag api are merged into update_state
    pub diff_ops: Vec<DiffOpsForSs>,
    // diff_ops list used for apply. which know about tags
    #[serde(skip_serializing)]
    pub detail_diff_ops: Vec<DiffOpsForSs>,
    pub no_change: bool,
    pub summary: HashMap<OpName, usize>,
}

impl Default for DiffResult {
    fn default() -> Self {
        DiffResult {
            text_diff: vec![],
            diff_ops: vec![],
            detail_diff_ops: vec![],
            no_change: true,
            summary: HashMap::from([
                ("create_state".to_string(), 0),
                ("update_state".to_string(), 0),
                ("delete_state".to_string(), 0),
                ("create_schedule".to_string(), 0),
                ("update_schedule".to_string(), 0),
                ("delete_schedule".to_string(), 0),
            ]),
        }
    }
}

impl DiffResult {
    pub fn append_text_diff(&mut self, diff: String) {
        let stripped_diff = strip_ansi_codes(&diff);
        self.text_diff.push(stripped_diff.to_string());
    }

    pub fn append_diff_op(&mut self, state_name: &str, diff_op: &DiffOp) {
        self.add_detail_diff_op(state_name, diff_op);
        self.add_diff_op(state_name, diff_op);

        self.no_change = false;
    }

    fn add_detail_diff_op(&mut self, state_name: &str, diff_op: &DiffOp) {
        let state_name = state_name.to_string();

        let diff_op_for_ss = self
            .detail_diff_ops
            .iter_mut()
            .find(|ddo| ddo.state_name == state_name);
        if let Some(diff_op_for_ss) = diff_op_for_ss {
            diff_op_for_ss.diff_ops.push(diff_op.clone());
        } else {
            let diff_op_for_ss = DiffOpsForSs {
                state_name,
                diff_ops: vec![diff_op.clone()],
            };
            self.detail_diff_ops.push(diff_op_for_ss);
        }
    }

    fn add_diff_op(&mut self, state_name: &str, diff_op: &DiffOp) {
        let state_name = state_name.to_string();
        let diff_op = DiffOp::op_for_report(diff_op);

        let diff_op_for_ss = self
            .diff_ops
            .iter_mut()
            .find(|ddo| ddo.state_name == state_name);
        if let Some(diff_op_for_ss) = diff_op_for_ss {
            if !diff_op_for_ss.diff_ops.contains(diff_op) {
                diff_op_for_ss.diff_ops.push(diff_op.clone());

                self.add_summary(diff_op);
            }
        } else {
            let diff_op_for_ss = DiffOpsForSs {
                state_name,
                diff_ops: vec![diff_op.clone()],
            };
            self.diff_ops.push(diff_op_for_ss);
            self.add_summary(diff_op);
        }
    }

    fn add_summary(&mut self, diff_op: &DiffOp) {
        self.summary
            .entry(diff_op.op_type().to_string())
            .and_modify(|e| *e += 1);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_summary() {
        let mut actual = DiffResult::default();

        actual.append_diff_op("NewBatch", &DiffOp::CreateState);
        actual.append_diff_op("HelloWorld", &DiffOp::UpdateState);
        actual.append_diff_op("HelloWorld", &DiffOp::AddStateTag);
        actual.append_diff_op(
            "HelloWorld",
            &DiffOp::RemoveStateTag(vec!["tag".to_string()]),
        );
        actual.append_diff_op("HelloWorld", &DiffOp::CreateSchedule);

        let expected = HashMap::from([
            ("create_state".to_string(), 1),
            ("update_state".to_string(), 1),
            ("delete_state".to_string(), 0),
            ("create_schedule".to_string(), 1),
            ("update_schedule".to_string(), 0),
            ("delete_schedule".to_string(), 0),
        ]);

        similar_asserts::assert_eq!(expected, actual.summary);
    }

    #[tokio::test]
    async fn test_no_change_flag() {
        let mut actual = DiffResult::default();

        similar_asserts::assert_eq!(true, actual.no_change);

        actual.append_diff_op("NewBatch", &DiffOp::CreateState);
        similar_asserts::assert_eq!(false, actual.no_change);
    }
}
