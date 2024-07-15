use std::collections::HashSet;

use console::Style;
use similar::{ChangeTag, TextDiff};

use crate::types::{DiffOp, ResourceTag, Schedule, SsConfig, StateMachine};

fn print_resource_diff(target: &str, remote: &str, local: &str) {
    let remote_name = format!("{} (remote)", target);
    let local_name = format!("{} (local)", target);

    let diff = TextDiff::from_lines(remote, local);

    for hunk in diff.unified_diff().missing_newline_hint(true).iter_hunks() {
        println!("--- {}", &remote_name);
        println!("+++ {}", &local_name);

        for change in hunk.iter_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
                ChangeTag::Delete => ("-", Style::new().red()),
            };
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }
    }
}

pub fn print_config_diff(
    local_config: &SsConfig,
    remote_state: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
    diff_ops: &[DiffOp],
) {
    let mut change_state = false;
    let mut delete_state = false;
    let mut change_schedule = false;
    let mut delete_schedule = false;

    for op in diff_ops {
        match op {
            DiffOp::CreateState
            | DiffOp::UpdateState
            | DiffOp::AddStateTag
            | DiffOp::RemoteStateTag(_) => {
                change_state = true;
            }
            DiffOp::DeleteState => {
                delete_state = true;
            }
            DiffOp::CreateSchedule | DiffOp::UpdateSchedule => {
                change_schedule = true;
            }
            DiffOp::DeleteSchedule => {
                delete_schedule = true;
            }
        }
    }

    if !change_state && !change_schedule && !delete_state && !delete_schedule {
        println!("no difference");
        return;
    }

    let target_state_id = local_config.state.name.as_str();
    if change_state {
        let remote_state_json_string = serde_json::to_string_pretty(&remote_state).unwrap();
        let local_state_json_string = serde_json::to_string_pretty(&local_config.state).unwrap();

        print_resource_diff(
            target_state_id,
            &remote_state_json_string,
            &local_state_json_string,
        );
    } else if delete_state {
        println!(
            "remote state machine({}) is going to be deleted",
            remote_state.as_ref().unwrap().name
        );
    }

    if change_schedule {
        let target_schedule_name = local_config.schedule.as_ref().unwrap().name.as_str();
        let remote_schedule_json_string = serde_json::to_string_pretty(&remote_schedule).unwrap();
        let local_schedule_json_string =
            serde_json::to_string_pretty(&local_config.schedule).unwrap();

        print_resource_diff(
            target_schedule_name,
            &remote_schedule_json_string,
            &local_schedule_json_string,
        );
    } else if delete_schedule {
        println!(
            "remote schedule({}) is going to be deleted",
            remote_schedule.as_ref().unwrap().name
        );
    }
}

fn split_sfn_and_tags(sfn: Option<StateMachine>) -> (Option<StateMachine>, Vec<ResourceTag>) {
    if sfn.is_none() {
        return (None, vec![]);
    }

    let mut sfn = sfn.unwrap();
    let tags = sfn.tags;
    sfn.tags = vec![];

    (Some(sfn), tags)
}

pub fn build_diff_ops(
    local_config: &SsConfig,
    remote_state: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
) -> Vec<DiffOp> {
    let mut expected_ops = vec![];

    let local_state = local_config.state.clone();
    let remote_state = remote_state.clone();

    let (local_state, local_state_tags) = split_sfn_and_tags(Some(local_state));
    let local_state = local_state.unwrap();
    let (remote_state, remote_state_tags) = split_sfn_and_tags(remote_state);

    if local_config.delete_all {
        if remote_state.is_some() {
            expected_ops.push(DiffOp::DeleteState);
        } else {
            // No change
        }
    } else if let Some(remote_state) = remote_state {
        if local_state != remote_state {
            expected_ops.push(DiffOp::UpdateState);
        } else {
            // No change
        }
    } else {
        expected_ops.push(DiffOp::CreateState);
    }

    if !expected_ops.contains(&DiffOp::DeleteState) {
        build_sfn_tags_diff_ops(&local_state_tags, &remote_state_tags, &mut expected_ops);
    }

    let local_schedule = local_config.schedule.clone();
    let remote_schedule = remote_schedule.clone();

    if local_config.delete_all || local_config.delete_schedule {
        if local_schedule.is_none() {
            panic!("delete schedule flag(deleteAll or deleteSchedule) is on, but can't identify schedule since schedule config is not exist.");
        }

        if remote_schedule.is_some() {
            expected_ops.push(DiffOp::DeleteSchedule);
        } else {
            // No change
        }
    } else if local_schedule.is_none() {
        // No change
    } else {
        let local_schedule = local_schedule.unwrap();

        if let Some(remote_schedule) = remote_schedule {
            if local_schedule == remote_schedule {
                // No change
            } else {
                expected_ops.push(DiffOp::UpdateSchedule);
            }
        } else {
            expected_ops.push(DiffOp::CreateSchedule);
        }
    }

    expected_ops
}

fn build_sfn_tags_diff_ops(
    local_state_tags: &[ResourceTag],
    remote_state_tags: &[ResourceTag],
    expected_ops: &mut Vec<DiffOp>,
) {
    let mut required_ops: HashSet<DiffOp> = HashSet::from([]);
    if local_state_tags == remote_state_tags {
        return;
    }

    let local_tag_keys: HashSet<&String> = local_state_tags.iter().map(|tag| &tag.key).collect();
    let remote_tag_keys: HashSet<&String> = remote_state_tags.iter().map(|tag| &tag.key).collect();

    let added_tag_keys: HashSet<_> = local_tag_keys.difference(&remote_tag_keys).collect();
    if !added_tag_keys.is_empty() {
        required_ops.insert(DiffOp::AddStateTag);
    }

    let removed_tag_keys: HashSet<_> = remote_tag_keys.difference(&local_tag_keys).collect();
    if !removed_tag_keys.is_empty() {
        let keys: Vec<String> = removed_tag_keys
            .into_iter()
            .map(|key| key.to_string())
            .collect();
        required_ops.insert(DiffOp::RemoteStateTag(keys));
    }

    let retained_tag_keys: HashSet<_> = local_tag_keys.intersection(&remote_tag_keys).collect();
    let local_retained_tags: Vec<_> = local_state_tags
        .iter()
        .filter(|tag| retained_tag_keys.contains(&&tag.key))
        .collect();
    let remote_retained_tags: Vec<_> = remote_state_tags
        .iter()
        .filter(|tag| retained_tag_keys.contains(&&tag.key))
        .collect();
    if local_retained_tags != remote_retained_tags {
        required_ops.insert(DiffOp::AddStateTag);
    }

    for op in required_ops {
        expected_ops.push(op);
    }
}

pub fn classify_diff_op(diff_op: &DiffOp) -> String {
    match diff_op {
        DiffOp::CreateState => "create_state",
        DiffOp::UpdateState => "update_state",
        DiffOp::DeleteState => "delete_state",
        DiffOp::AddStateTag => "update_state",
        DiffOp::RemoteStateTag(_) => "update_state",
        DiffOp::CreateSchedule => "create_schedule",
        DiffOp::UpdateSchedule => "update_schedule",
        DiffOp::DeleteSchedule => "delete_schedule",
    }
    .to_string()
}

pub fn print_diff_ops(diff_ops: &[DiffOp]) {
    if diff_ops.is_empty() {
        println!("no difference");
        return;
    }

    let mut op_counts = std::collections::HashSet::new();
    for op in diff_ops.iter() {
        let class = classify_diff_op(op);
        op_counts.insert(class);
    }

    println!("diff ops:");
    for op in op_counts.iter() {
        println!("{}: 1", op);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_tags_diff_ops_with_no_diff() {
        let mut actual_ops: Vec<DiffOp> = vec![];

        let local_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];
        let remote_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];

        build_sfn_tags_diff_ops(&local_tags, &remote_tags, &mut actual_ops);

        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_build_tags_diff_ops_with_add_tag() {
        let mut actual_ops: Vec<DiffOp> = vec![];

        let local_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];
        let remote_tags = vec![ResourceTag {
            key: "key1".to_string(),
            value: "value1".to_string(),
        }];

        build_sfn_tags_diff_ops(&local_tags, &remote_tags, &mut actual_ops);

        assert_eq!(actual_ops, vec![DiffOp::AddStateTag]);
    }

    #[test]
    fn test_build_tags_diff_ops_with_remove_tag() {
        let mut actual_ops: Vec<DiffOp> = vec![];

        let local_tags = vec![ResourceTag {
            key: "key1".to_string(),
            value: "value1".to_string(),
        }];
        let remote_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];

        build_sfn_tags_diff_ops(&local_tags, &remote_tags, &mut actual_ops);

        assert_eq!(
            actual_ops,
            vec![DiffOp::RemoteStateTag(vec!["key2".to_string()])]
        );
    }

    #[test]
    fn test_build_tags_diff_ops_with_rewrite_tag() {
        let mut actual_ops: Vec<DiffOp> = vec![];

        let local_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "new_value".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];
        let remote_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
        ];

        build_sfn_tags_diff_ops(&local_tags, &remote_tags, &mut actual_ops);

        assert_eq!(actual_ops, vec![DiffOp::AddStateTag]);
    }

    #[test]
    fn test_build_tags_diff_ops_with_composite_tag() {
        let mut actual_ops: Vec<DiffOp> = vec![];

        let local_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "new_value".to_string(),
            },
            ResourceTag {
                key: "local_only_key".to_string(),
                value: "local_only_value".to_string(),
            },
        ];
        let remote_tags = vec![
            ResourceTag {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
            ResourceTag {
                key: "key2".to_string(),
                value: "value2".to_string(),
            },
            ResourceTag {
                key: "remote_only_key".to_string(),
                value: "remote_only_value".to_string(),
            },
        ];

        build_sfn_tags_diff_ops(&local_tags, &remote_tags, &mut actual_ops);
        actual_ops.sort();

        assert_eq!(
            actual_ops,
            vec![
                DiffOp::AddStateTag,
                DiffOp::RemoteStateTag(vec!["remote_only_key".to_string()])
            ]
        );
    }
}
