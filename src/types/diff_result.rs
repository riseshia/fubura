use std::collections::HashMap;

use serde::{Deserialize, Serialize};

type OpName = String;
type StateName = String;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct OpWithName {
    pub op_name: OpName,
    pub list: Vec<StateName>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DiffResult {
    pub text_diff: Vec<String>,
    pub diff_ops: Vec<OpWithName>,
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
            diff_ops: vec![
                OpWithName {
                    op_name: "create_state".to_string(),
                    list: vec![],
                },
                OpWithName {
                    op_name: "update_state".to_string(),
                    list: vec![],
                },
                OpWithName {
                    op_name: "delete_state".to_string(),
                    list: vec![],
                },
                OpWithName {
                    op_name: "create_schedule".to_string(),
                    list: vec![],
                },
                OpWithName {
                    op_name: "update_schedule".to_string(),
                    list: vec![],
                },
                OpWithName {
                    op_name: "delete_schedule".to_string(),
                    list: vec![],
                },
            ],
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
    fn enabled(&self) -> bool {
        self.json_diff_path.is_some()
    }

    pub fn append_text_diff(&mut self, diff: String) {
        if !self.enabled() {
            return;
        }

        self.text_diff.push(diff);
    }

    pub fn append_diff_op(&mut self, op_name: &OpName, state_name: &StateName) {
        if !self.enabled() {
            return;
        }

        let op_name = op_name.clone();
        let state_name = state_name.clone();

        let diff_op = self
            .diff_ops
            .iter_mut()
            .find(|op| op.op_name == op_name)
            .unwrap_or_else(|| panic!("op_name: {} not found in diff_ops", op_name));

        diff_op.list.push(state_name);
        self.summary.entry(op_name).and_modify(|e| *e += 1);
        self.no_change = false;
    }
}
