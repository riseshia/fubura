use std::collections::HashSet;

use similar::TextDiff;

use crate::types::{DiffOp, ResourceTag, Schedule, SsConfig, StateMachine};

fn print_resource_diff(remote: &str, local: &str) {
    let diff = TextDiff::from_lines(remote, local);

    diff.unified_diff()
        .header("remote", "local")
        .to_writer(std::io::stdout())
        .unwrap();
}

pub fn print_config_diff(
    local_config: &SsConfig,
    remote_state: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
) {
    let remote_state_json_string = serde_json::to_string_pretty(&remote_state).unwrap();
    let local_state_json_string = serde_json::to_string_pretty(&local_config.state).unwrap();

    let mut has_no_diff = true;

    if local_state_json_string != remote_state_json_string {
        has_no_diff = false;
        print_resource_diff(&remote_state_json_string, &local_state_json_string);
    }

    if let Some(local_schedule) = &local_config.schedule {
        let local_schedule_json_string = serde_json::to_string_pretty(&local_schedule).unwrap();

        match remote_schedule {
            Some(remote_schedule) => {
                let remote_schedule_json_string =
                    serde_json::to_string_pretty(&remote_schedule).unwrap();

                if remote_schedule_json_string != local_schedule_json_string {
                    has_no_diff = false;
                    print_resource_diff(&remote_schedule_json_string, &local_schedule_json_string);
                }
            }
            None => {
                print_resource_diff("null", &local_schedule_json_string);
            }
        }
    } else {
        todo!("no local schedule & remote schedule exists/not exists case");
    }

    if has_no_diff {
        println!("no difference");
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

    if local_config.delete_state {
        if remote_state.is_some() {
            expected_ops.push(DiffOp::DeleteSfn);
        } else {
            expected_ops.push(DiffOp::NoChangeSfn);
        }
    } else if let Some(remote_state) = remote_state {
        if local_state != remote_state {
            expected_ops.push(DiffOp::UpdateSfn);
        } else {
            expected_ops.push(DiffOp::NoChangeSfn);
        }
    } else {
        expected_ops.push(DiffOp::CreateSfn);
    }

    build_sfn_tags_diff_ops(&local_state_tags, &remote_state_tags, &mut expected_ops);

    let local_schedule = local_config.schedule.clone();
    let remote_schedule = remote_schedule.clone();

    if local_config.delete_schedule {
        if local_schedule.is_none() {
            panic!("delete_schedule flag is on, but can't identify schedule since schedule config is not exist.");
        }

        if remote_schedule.is_some() {
            expected_ops.push(DiffOp::DeleteSchedule);
        } else {
            expected_ops.push(DiffOp::NoChangeSchedule);
        }
    } else if local_schedule.is_none() {
        expected_ops.push(DiffOp::NoChangeSchedule);
    } else {
        let local_schedule = local_schedule.unwrap();

        if let Some(remote_schedule) = remote_schedule {
            if local_schedule == remote_schedule {
                expected_ops.push(DiffOp::NoChangeSchedule);
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
        required_ops.insert(DiffOp::AddSfnTag);
    }

    let removed_tag_keys: HashSet<_> = remote_tag_keys.difference(&local_tag_keys).collect();
    if !removed_tag_keys.is_empty() {
        required_ops.insert(DiffOp::RemoveSfnTag);
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
        required_ops.insert(DiffOp::AddSfnTag);
    }

    for op in required_ops {
        expected_ops.push(op);
    }
}

fn classify_diff_op(diff_op: &DiffOp) -> String {
    match diff_op {
        DiffOp::CreateSfn => "create",
        DiffOp::UpdateSfn => "update",
        DiffOp::DeleteSfn => "delete",
        DiffOp::NoChangeSfn => "no change",
        DiffOp::AddSfnTag => "create",
        DiffOp::RemoveSfnTag => "update",
        DiffOp::NoChangeSfnTags => "no change",
        DiffOp::CreateSchedule => "create",
        DiffOp::UpdateSchedule => "update",
        DiffOp::DeleteSchedule => "delete",
        DiffOp::NoChangeSchedule => "no change",
    }
    .to_string()
}

pub fn print_diff_ops(diff_ops: &[DiffOp]) {
    if diff_ops.is_empty() {
        println!("no difference");
        return;
    }

    let mut op_counts = std::collections::HashMap::new();
    for op in diff_ops.iter() {
        let class = classify_diff_op(op);
        let count = op_counts.entry(class).or_insert(0);
        *count += 1;
    }

    println!("diff ops:");
    for (op, count) in op_counts.iter() {
        println!("{:?}: {}", op, count);
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

        assert_eq!(actual_ops, vec![DiffOp::AddSfnTag]);
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

        assert_eq!(actual_ops, vec![DiffOp::RemoveSfnTag]);
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

        assert_eq!(actual_ops, vec![DiffOp::AddSfnTag]);
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

        assert_eq!(actual_ops, vec![DiffOp::AddSfnTag, DiffOp::RemoveSfnTag]);
    }
}
