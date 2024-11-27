use std::collections::HashSet;

use anyhow::{bail, Result};
use console::Style;
use similar::{ChangeTag, TextDiff};
use tracing::{debug, info};

use crate::{
    context::FuburaContext,
    scheduler, sfn, sts,
    types::{Config, DiffOp, DiffResult, ResourceTag, Schedule, SsConfig, StateMachine},
};

fn format_resource_diff(target: &str, remote: &str, local: &str) -> String {
    let mut buffer = String::new();

    let remote_name = format!("remote: {}", target);
    let local_name = format!("local:  {}", target);

    let diff = TextDiff::from_lines(remote, local);

    for hunk in diff.unified_diff().missing_newline_hint(true).iter_hunks() {
        buffer.push_str(&format!("--- {}\n", &remote_name));
        buffer.push_str(&format!("+++ {}\n", &local_name));

        for change in hunk.iter_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
                ChangeTag::Delete => ("-", Style::new().red()),
            };
            buffer.push_str(&format!(
                "{}{}",
                style.apply_to(sign).bold(),
                style.apply_to(change)
            ));
        }
    }

    buffer
}

fn format_config_diff(
    local_config: &SsConfig,
    remote_state: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
    diff_ops: &[DiffOp],
) -> Option<String> {
    let mut change_state = false;
    let mut delete_state = false;
    let mut change_schedule = false;
    let mut delete_schedule = false;

    let mut buffer = String::new();

    for op in diff_ops {
        match op {
            DiffOp::CreateState
            | DiffOp::UpdateState
            | DiffOp::AddStateTag
            | DiffOp::RemoveStateTag(_) => {
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
        return None;
    }

    let target_state_id = local_config.state.name.as_str();
    if change_state {
        let remote_state_json_string = serde_json::to_string_pretty(&remote_state).unwrap();
        let local_state_json_string = serde_json::to_string_pretty(&local_config.state).unwrap();

        let text_diff = format_resource_diff(
            target_state_id,
            &remote_state_json_string,
            &local_state_json_string,
        );
        buffer.push_str(format!("{}\n", text_diff).as_str());
    } else if delete_state {
        let str = format!(
            "State machine({}) is going to be deleted\n",
            remote_state.as_ref().unwrap().name
        );
        buffer.push_str(str.as_str());
    }

    if change_schedule {
        let target_schedule_name = local_config.schedule.as_ref().unwrap().name.as_str();
        let remote_schedule_json_string = serde_json::to_string_pretty(&remote_schedule).unwrap();
        let local_schedule_json_string =
            serde_json::to_string_pretty(&local_config.schedule).unwrap();

        let text_diff = format_resource_diff(
            target_schedule_name,
            &remote_schedule_json_string,
            &local_schedule_json_string,
        );
        let str = format!("{}\n", text_diff);
        buffer.push_str(str.as_str());
    } else if delete_schedule {
        let str = format!(
            "Schedule({}) is going to be deleted\n",
            remote_schedule.as_ref().unwrap().name
        );
        buffer.push_str(str.as_str());
    }

    Some(buffer)
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

fn build_diff_ops(
    local_config: &SsConfig,
    remote_state: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
) -> Result<Vec<DiffOp>> {
    let mut expected_ops = vec![];

    let local_state = local_config.state.clone();
    let remote_state = remote_state.clone();

    let (local_state, local_state_tags) = split_sfn_and_tags(Some(local_state));
    let local_state = local_state.unwrap();
    let (remote_state, remote_state_tags) = split_sfn_and_tags(remote_state);

    let local_schedule = local_config.schedule.clone();
    let remote_schedule = remote_schedule.clone();

    if local_config.delete_all {
        if local_schedule.is_some() && remote_schedule.is_some() {
            expected_ops.push(DiffOp::DeleteSchedule);
        } else {
            // No change
        }

        if remote_state.is_some() {
            expected_ops.push(DiffOp::DeleteState);
        } else {
            // No change
        }

        return Ok(expected_ops);
    }

    if local_config.delete_schedule {
        if local_schedule.is_none() {
            bail!("delete schedule flag(deleteSchedule) is on, but can't identify schedule since schedule config is not exist.");
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

    if let Some(remote_state) = remote_state {
        if local_state != remote_state {
            expected_ops.push(DiffOp::UpdateState);
        } else {
            // No change
        }
    } else {
        expected_ops.push(DiffOp::CreateState);
        expected_ops.sort();

        // Create ops will handle tags as it is, so dont need to push tags ops, so return here
        return Ok(expected_ops);
    }

    // Create ops will handle tags as it is, so dont need to push tags ops
    if !expected_ops.contains(&DiffOp::DeleteState) {
        let ops = build_sfn_tags_diff_ops(&local_state_tags, &remote_state_tags);
        for op in ops {
            expected_ops.push(op);
        }
    }

    expected_ops.sort();
    Ok(expected_ops)
}

fn build_sfn_tags_diff_ops(
    local_state_tags: &[ResourceTag],
    remote_state_tags: &[ResourceTag],
) -> Vec<DiffOp> {
    let mut required_ops: HashSet<DiffOp> = HashSet::from([]);
    if local_state_tags == remote_state_tags {
        return vec![];
    }

    let local_tag_keys: HashSet<&String> = local_state_tags.iter().map(|tag| &tag.key).collect();
    let remote_tag_keys: HashSet<&String> = remote_state_tags.iter().map(|tag| &tag.key).collect();

    // Check added
    let added_tag_keys: HashSet<_> = local_tag_keys.difference(&remote_tag_keys).collect();
    if !added_tag_keys.is_empty() {
        required_ops.insert(DiffOp::AddStateTag);
    }

    // Check value updated
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

    // Check deleted
    let removed_tag_keys: HashSet<_> = remote_tag_keys.difference(&local_tag_keys).collect();
    if !removed_tag_keys.is_empty() {
        let keys: Vec<String> = removed_tag_keys
            .into_iter()
            .map(|key| key.to_string())
            .collect();
        required_ops.insert(DiffOp::RemoveStateTag(keys));
    }

    let mut diff_ops: Vec<DiffOp> = required_ops.into_iter().collect();
    diff_ops.sort();
    diff_ops
}

// Sort remote tags by local tags order
// to make sure the diff result is consistent
pub fn sort_tags_by_local_tags_order(
    remote_state: Option<StateMachine>,
    local_tags: &[ResourceTag],
) -> Option<StateMachine> {
    if remote_state.is_none() {
        return remote_state;
    }
    let mut remote_state = remote_state.unwrap();
    let remote_tags = remote_state.tags;

    let local_tag_keys: HashSet<&String> = local_tags.iter().map(|tag| &tag.key).collect();
    let remote_tag_keys: HashSet<&String> = remote_tags.iter().map(|tag| &tag.key).collect();
    let remote_only_tag_keys = remote_tag_keys.difference(&local_tag_keys);
    let mut sorted_remote_tags = vec![];

    for local_tag in local_tags {
        if let Some(remote_tag) = remote_tags.iter().find(|tag| tag.key == local_tag.key) {
            sorted_remote_tags.push(remote_tag.clone());
        }
    }

    for tag_key in remote_only_tag_keys {
        if let Some(tag) = remote_tags.iter().find(|tag| tag.key == **tag_key) {
            sorted_remote_tags.push(tag.clone());
        }
    }

    remote_state.tags = sorted_remote_tags;

    Some(remote_state)
}

pub async fn diff(context: &FuburaContext, config: &Config) -> Result<DiffResult> {
    let mut diff_result = DiffResult::default();

    let state_arn_prefix = sts::build_state_arn_prefix(context).await;

    let target_ss_configs = config.target_ss_configs(&context.targets);

    for ss_config in target_ss_configs {
        let state_arn = format!("{}{}", state_arn_prefix, ss_config.state.name);

        info!("Describing state machine: {}", &state_arn);
        let remote_state =
            sfn::describe_state_machine_with_tags(&context.sfn_client, &state_arn).await?;
        let remote_state = sort_tags_by_local_tags_order(remote_state, &ss_config.state.tags);

        info!("Describing schedule: {}", &state_arn);
        let remote_schedule = if let Some(schedule_config) = &ss_config.schedule {
            scheduler::get_schedule(
                &context.scheduler_client,
                &schedule_config.schedule_name_with_group(),
            )
            .await?
        } else {
            None
        };

        let diff_ops = build_diff_ops(ss_config, &remote_state, &remote_schedule)?;
        debug!("state machine name: {}", &ss_config.state.name);
        debug!("generated diff ops: {:?}", &diff_ops);

        for diff_op in diff_ops.iter() {
            diff_result.append_diff_op(&ss_config.state.name, diff_op)
        }

        let text_diff = format_config_diff(ss_config, &remote_state, &remote_schedule, &diff_ops);
        if let Some(text_diff) = text_diff {
            println!("{}", text_diff);
            diff_result.append_text_diff(text_diff);
        } else {
            println!("no difference");
            // do not append empty diff which is too verbose
        }
    }

    if diff_result.no_change {
        println!("\nNo diff found. Fubura will do nothing.");
    } else {
        println!("\nFubura will:");
        for (op, count) in diff_result.summary.iter() {
            println!("    {}: {}", op, count);
        }
    }

    Ok(diff_result)
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    use crate::types::{DiffOp, Schedule, SsConfig, StateMachine};

    use aws_sdk_scheduler::operation::get_schedule::GetScheduleError;
    use aws_sdk_scheduler::{
        operation::get_schedule::builders::GetScheduleOutputBuilder, types::builders::TargetBuilder,
    };
    use aws_sdk_sfn::error::SdkError;
    use aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineError;
    use aws_sdk_sfn::operation::list_tags_for_resource::builders::ListTagsForResourceOutputBuilder;
    use aws_sdk_sfn::types::builders::{
        CloudWatchLogsLogGroupBuilder, LogDestinationBuilder, LoggingConfigurationBuilder,
        TagBuilder,
    };
    use aws_sdk_sfn::types::LogLevel;
    use aws_sdk_sfn::{
        operation::describe_state_machine::builders::DescribeStateMachineOutputBuilder,
        primitives::{DateTime, DateTimeFormat},
        types::StateMachineType,
    };
    use aws_sdk_sts::operation::get_caller_identity::builders::GetCallerIdentityOutputBuilder;

    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;
    use mockall::predicate::eq;

    #[test]
    fn test_build_diff_ops_returns_no_diff() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_build_diff_ops_returns_create_state() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: None,
            delete_all: false,
            delete_schedule: false,
        };

        let remote_state = None;
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::CreateState]);
    }

    #[test]
    fn test_build_diff_ops_returns_create_state_and_schedule() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };

        let remote_state = None;
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![DiffOp::CreateState, DiffOp::CreateSchedule]
        );
    }

    #[test]
    fn test_build_diff_ops_returns_update_state() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };

        let mut remote_state = StateMachine::test_default();
        remote_state.definition = json!({
            "StartAt": "Updated",
        });
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::UpdateState]);
    }

    #[test]
    fn test_build_diff_ops_returns_update_state_and_add_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.push(ResourceTag {
            key: "new_key".to_string(),
            value: "value".to_string(),
        });

        let mut remote_state = StateMachine::test_default();
        remote_state.definition = json!({
            "StartAt": "Updated",
        });
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::UpdateState, DiffOp::AddStateTag]);
    }

    #[test]
    fn test_build_diff_ops_returns_update_state_and_remove_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.pop();

        let mut remote_state = StateMachine::test_default();
        remote_state.definition = json!({
            "StartAt": "Updated",
        });
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![
                DiffOp::UpdateState,
                DiffOp::RemoveStateTag(vec!["Name".to_string()])
            ]
        );
    }

    #[test]
    fn test_build_diff_ops_returns_update_state_and_add_remove_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.pop();
        local_config.state.tags.push(ResourceTag {
            key: "new_key".to_string(),
            value: "value".to_string(),
        });

        let mut remote_state = StateMachine::test_default();
        remote_state.definition = json!({
            "StartAt": "Updated",
        });
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![
                DiffOp::UpdateState,
                DiffOp::AddStateTag,
                DiffOp::RemoveStateTag(vec!["Name".to_string()])
            ]
        );
    }

    #[test]
    fn test_build_diff_ops_returns_add_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.push(ResourceTag {
            key: "new_key".to_string(),
            value: "value".to_string(),
        });

        let remote_state = StateMachine::test_default();
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::AddStateTag]);
    }

    #[test]
    fn test_build_diff_ops_returns_remove_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.pop();

        let remote_state = StateMachine::test_default();
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![DiffOp::RemoveStateTag(vec!["Name".to_string()])]
        );
    }

    #[test]
    fn test_build_diff_ops_returns_add_remove_tag() {
        let mut local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };
        local_config.state.tags.pop();
        local_config.state.tags.push(ResourceTag {
            key: "new_key".to_string(),
            value: "value".to_string(),
        });

        let remote_state = StateMachine::test_default();
        let remote_state = Some(remote_state);

        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![
                DiffOp::AddStateTag,
                DiffOp::RemoveStateTag(vec!["Name".to_string()])
            ]
        );
    }

    #[test]
    fn test_build_diff_ops_returns_update_schedule() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: false,
        };

        let remote_state = Some(StateMachine::test_default());

        let mut remote_schedule = Schedule::test_default();
        remote_schedule.schedule_expression = "rate(1 hour)".to_string();
        let remote_schedule = Some(remote_schedule);

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::UpdateSchedule]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_schedule_flag_returns_delete_schedule() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: true,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::DeleteSchedule]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_schedule_flag_returns_no_diff() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: false,
            delete_schedule: true,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_schedule_flag_returns_error() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: None,
            delete_all: false,
            delete_schedule: true,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule);
        assert!(actual_ops.is_err());
        let err = actual_ops.unwrap_err();
        assert_eq!(
            err.to_string(),
            "delete schedule flag(deleteSchedule) is on, but can't identify schedule since schedule config is not exist."
        );
    }

    #[test]
    fn test_build_diff_ops_with_delete_all_flag_returns_delete_state_and_schedule() {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: true,
            delete_schedule: false,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = Some(Schedule::test_default());

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(
            actual_ops,
            vec![DiffOp::DeleteSchedule, DiffOp::DeleteState]
        );
    }

    #[test]
    fn test_build_diff_ops_with_delete_all_flag_returns_deletes_state_when_local_schedule_exist_but_remote_not(
    ) {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: true,
            delete_schedule: false,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::DeleteState]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_all_flag_returns_deletes_state_when_local_and_remote_schedule_not_exist(
    ) {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: None,
            delete_all: true,
            delete_schedule: false,
        };

        let remote_state = Some(StateMachine::test_default());
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![DiffOp::DeleteState]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_all_flag_returns_no_diff_when_local_resource_exist_but_remote_resources_not_exist(
    ) {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: Some(Schedule::test_default()),
            delete_all: true,
            delete_schedule: false,
        };

        let remote_state = None;
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_build_diff_ops_with_delete_all_flag_returns_no_diff_when_local_state_exist_but_remote_state_not_exist(
    ) {
        let local_config = SsConfig {
            state: StateMachine::test_default(),
            schedule: None,
            delete_all: true,
            delete_schedule: false,
        };

        let remote_state = None;
        let remote_schedule = None;

        let actual_ops = build_diff_ops(&local_config, &remote_state, &remote_schedule).unwrap();

        assert_eq!(actual_ops, vec![]);
    }

    #[tokio::test]
    async fn test_diff_no_diff() {
        let mut context = FuburaContext::async_default().await;

        context
            .sts_client
            .expect_get_caller_identity()
            .return_once(|| {
                Ok(GetCallerIdentityOutputBuilder::default()
                    .account("123456789012".to_string())
                    .build())
            });

        context
            .sfn_client
            .expect_describe_state_machine()
            .with(eq(
                "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            ))
            .return_once(|_| {
                Ok(DescribeStateMachineOutputBuilder::default()
                    .state_machine_arn(
                        "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
                    )
                    .name("HelloWorld".to_string())
                    .r#type(StateMachineType::Standard)
                    .definition("{ \"StartAt\": \"FirstState\" }".to_string())
                    .role_arn(
                        "arn:aws:iam::123456789012:role/service-role/HelloWorldRole".to_string(),
                    )
                    .logging_configuration(
                        LoggingConfigurationBuilder::default()
                            .level(LogLevel::All)
                            .include_execution_data(true)
                            .destinations(
                                LogDestinationBuilder::default()
                                    .cloud_watch_logs_log_group(
                                        CloudWatchLogsLogGroupBuilder::default()
                                            .log_group_arn("arn:aws:logs:us-west-2:123456789012:log-group:HelloWorldLogGroup")
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .creation_date(
                        DateTime::from_str("2021-01-01T00:00:00Z", DateTimeFormat::DateTime)
                            .unwrap(),
                    )
                    .build()
                    .unwrap())
            });

        context
            .sfn_client
            .expect_list_tags_for_resource()
            .with(eq(
                "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            ))
            .return_once(|_| {
                Ok(ListTagsForResourceOutputBuilder::default()
                    .tags(TagBuilder::default().key("Env").value("Test").build())
                    .tags(
                        TagBuilder::default()
                            .key("Name")
                            .value("HelloWorld")
                            .build(),
                    )
                    .build())
            });

        context
            .scheduler_client
            .expect_get_schedule()
            .with(eq("default"), eq("HelloWorld"))
            .return_once(|_, _| {
                Ok(GetScheduleOutputBuilder::default()
                    .arn("arn:aws:scheduler:us-west-2:123456789012:schedule:default/HelloWorld")
                    .group_name("default")
                    .name("HelloWorld")
                    .description("HellowWorld schedule")
                    .schedule_expression("rate(1 minute)")
                    .schedule_expression_timezone("UTC")
                    .state(aws_sdk_scheduler::types::ScheduleState::Enabled)
                    .target(
                        TargetBuilder::default()
                            .arn("arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld")
                            .role_arn("arn:aws:iam::123456789012:role/service-role/HelloWorldRole")
                            .build()
                            .unwrap(),
                    )
                    .build())
            });

        let config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: Some(Schedule::test_default()),
                delete_all: false,
                delete_schedule: false,
            }],
        };

        let diff_result = diff(&context, &config).await.unwrap();
        assert!(diff_result.no_change);
    }

    #[tokio::test]
    async fn test_diff_no_diff_with_different_tag_order() {
        let mut context = FuburaContext::async_default().await;

        context
            .sts_client
            .expect_get_caller_identity()
            .return_once(|| {
                Ok(GetCallerIdentityOutputBuilder::default()
                    .account("123456789012".to_string())
                    .build())
            });

        context
            .sfn_client
            .expect_describe_state_machine()
            .with(eq(
                "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            ))
            .return_once(|_| {
                Ok(DescribeStateMachineOutputBuilder::default()
                    .state_machine_arn(
                        "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
                    )
                    .name("HelloWorld".to_string())
                    .r#type(StateMachineType::Standard)
                    .definition("{ \"StartAt\": \"FirstState\" }".to_string())
                    .role_arn(
                        "arn:aws:iam::123456789012:role/service-role/HelloWorldRole".to_string(),
                    )
                    .logging_configuration(
                        LoggingConfigurationBuilder::default()
                            .level(LogLevel::All)
                            .include_execution_data(true)
                            .destinations(
                                LogDestinationBuilder::default()
                                    .cloud_watch_logs_log_group(
                                        CloudWatchLogsLogGroupBuilder::default()
                                            .log_group_arn("arn:aws:logs:us-west-2:123456789012:log-group:HelloWorldLogGroup")
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .creation_date(
                        DateTime::from_str("2021-01-01T00:00:00Z", DateTimeFormat::DateTime)
                            .unwrap(),
                    )
                    .build()
                    .unwrap())
            });

        context
            .sfn_client
            .expect_list_tags_for_resource()
            .with(eq(
                "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            ))
            .return_once(|_| {
                Ok(ListTagsForResourceOutputBuilder::default()
                    .tags(
                        TagBuilder::default()
                            .key("Name")
                            .value("HelloWorld")
                            .build(),
                    )
                    .tags(TagBuilder::default().key("Env").value("Test").build())
                    .build())
            });

        context
            .scheduler_client
            .expect_get_schedule()
            .with(eq("default"), eq("HelloWorld"))
            .return_once(|_, _| {
                Ok(GetScheduleOutputBuilder::default()
                    .arn("arn:aws:scheduler:us-west-2:123456789012:schedule:default/HelloWorld")
                    .group_name("default")
                    .name("HelloWorld")
                    .description("HellowWorld schedule")
                    .schedule_expression("rate(1 minute)")
                    .schedule_expression_timezone("UTC")
                    .state(aws_sdk_scheduler::types::ScheduleState::Enabled)
                    .target(
                        TargetBuilder::default()
                            .arn("arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld")
                            .role_arn("arn:aws:iam::123456789012:role/service-role/HelloWorldRole")
                            .build()
                            .unwrap(),
                    )
                    .build())
            });

        let config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: Some(Schedule::test_default()),
                delete_all: false,
                delete_schedule: false,
            }],
        };

        let diff_result = diff(&context, &config).await.unwrap();
        assert!(diff_result.no_change);
    }

    #[tokio::test]
    async fn test_create_state_and_schedule() {
        let mut context = FuburaContext::async_default().await;

        context
            .sts_client
            .expect_get_caller_identity()
            .return_once(|| {
                Ok(GetCallerIdentityOutputBuilder::default()
                    .account("123456789012".to_string())
                    .build())
            });

        context
            .sfn_client
            .expect_describe_state_machine()
            .with(eq(
                "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            ))
            .return_once(|_| {
                Err(SdkError::service_error(
                    DescribeStateMachineError::StateMachineDoesNotExist(
                        aws_sdk_sfn::types::error::StateMachineDoesNotExist::builder().build(),
                    ),
                    Response::new(StatusCode::try_from(404).unwrap(), SdkBody::empty()),
                ))
            });

        context
            .scheduler_client
            .expect_get_schedule()
            .with(eq("default"), eq("HelloWorld"))
            .return_once(|_, _| {
                Err(aws_sdk_scheduler::error::SdkError::service_error(
                    GetScheduleError::ResourceNotFoundException(
                        aws_sdk_scheduler::types::error::ResourceNotFoundException::builder()
                            .message("Resource not found")
                            .build()
                            .unwrap(),
                    ),
                    Response::new(StatusCode::try_from(404).unwrap(), SdkBody::empty()),
                ))
            });

        let config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: Some(Schedule::test_default()),
                delete_all: false,
                delete_schedule: false,
            }],
        };

        let mut actual_diff_result = diff(&context, &config).await.unwrap();
        actual_diff_result.text_diff.clear(); // do not check text_diff
        let mut expected_diff_result = DiffResult::default();
        expected_diff_result.append_diff_op("HelloWorld", &DiffOp::CreateState);
        expected_diff_result.append_diff_op("HelloWorld", &DiffOp::CreateSchedule);

        similar_asserts::assert_eq!(expected_diff_result, actual_diff_result);
    }
}
