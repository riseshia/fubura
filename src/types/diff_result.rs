use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::DiffOp;

type OpName = String;
type StateName = String;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct OpWithName {
    pub op_name: OpName,
    pub list: Vec<StateName>,
}

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
    pub detail_diff_ops: Vec<DiffOpsForSs>,
    pub no_change: bool,
    pub summary: HashMap<OpName, usize>,
    #[serde(skip_serializing)]
    pub json_diff_path: Option<String>,
}

impl Default for DiffResult {
    fn default() -> Self {
        DiffResult {
            json_diff_path: None,
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
        self.text_diff.push(diff);
    }

    pub fn append_diff_op(&mut self, state_name: &StateName, diff_op: &DiffOp) {
        self.add_detail_diff_op(state_name, diff_op);
        self.add_diff_op(state_name, diff_op);

        self.summary
            .entry(diff_op.op_type().to_string())
            .and_modify(|e| *e += 1);
        self.no_change = false;
    }

    fn add_detail_diff_op(&mut self, state_name: &StateName, diff_op: &DiffOp) {
        let state_name = state_name.clone();

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

    fn add_diff_op(&mut self, state_name: &StateName, diff_op: &DiffOp) {
        let state_name = state_name.clone();
        let diff_op = DiffOp::op_for_report(diff_op);

        let diff_op_for_ss = self
            .diff_ops
            .iter_mut()
            .find(|ddo| ddo.state_name == state_name);
        if let Some(diff_op_for_ss) = diff_op_for_ss {
            diff_op_for_ss.diff_ops.push(diff_op.clone());
        } else {
            let diff_op_for_ss = DiffOpsForSs {
                state_name,
                diff_ops: vec![diff_op.clone()],
            };
            self.diff_ops.push(diff_op_for_ss);
        }
    }
}
