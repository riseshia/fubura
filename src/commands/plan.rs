use anyhow::Result;

use crate::context::Context;
use crate::differ::{build_diff_ops, format_config_diff};
use crate::types::{Config, DiffResult};
use crate::{scheduler, sfn, sts};

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(context: &Context, config: &Config) -> Result<()> {
        plan(context, config).await?;
        Ok(())
    }
}

fn write_result_to_path(output_path: &str, diff_result: &DiffResult) {
    let json_diff = serde_json::to_string_pretty(diff_result).unwrap();
    std::fs::write(output_path, json_diff).unwrap_or_else(|_| {
        panic!("Failed to write diff result to {}", output_path);
    });
}

async fn plan(context: &Context, config: &Config) -> Result<DiffResult> {
    let mut diff_result = DiffResult::default();

    let state_arn_prefix = sts::build_state_arn_prefix(context).await;

    let target_ss_configs = config.target_ss_configs(&context.targets);

    for ss_config in target_ss_configs {
        let state_arn = format!("{}{}", state_arn_prefix, ss_config.state.name);

        let remote_state =
            sfn::describe_state_machine_with_tags(&context.sfn_client, &state_arn).await;

        let remote_schedule = if let Some(schedule_config) = &ss_config.schedule {
            scheduler::get_schedule(
                &context.scheduler_client,
                &schedule_config.schedule_name_with_group(),
            )
            .await
        } else {
            None
        };

        let diff_ops = build_diff_ops(ss_config, &remote_state, &remote_schedule);
        for diff_op in diff_ops.iter() {
            diff_result.append_diff_op(&ss_config.state.name, diff_op)
        }

        let text_diff = format_config_diff(ss_config, &remote_state, &remote_schedule, &diff_ops);
        println!("{}", &text_diff);
        diff_result.append_text_diff(text_diff);
    }

    println!("\nFubura will:");
    for (op, count) in diff_result.summary.iter() {
        println!("    {}: {}", op, count);
    }

    if let Some(json_diff_path) = &context.json_diff_path {
        write_result_to_path(json_diff_path, &diff_result);
    }

    Ok(diff_result)
}

#[cfg(test)]
mod test {
    use crate::types::{DiffOp, Schedule, SsConfig, StateMachine};

    use super::*;

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

    #[tokio::test]
    async fn test_no_diff() {
        let mut context = Context::async_default().await;

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

        let diff_result = plan(&context, &config).await.unwrap();
        assert!(diff_result.no_change);
    }

    #[tokio::test]
    async fn test_create_state_and_schedule() {
        let mut context = Context::async_default().await;

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

        let mut actual_diff_result = plan(&context, &config).await.unwrap();
        actual_diff_result.text_diff.clear(); // do not check text_diff
        let mut expected_diff_result = DiffResult::default();
        expected_diff_result.append_diff_op("HelloWorld", &DiffOp::CreateState);
        expected_diff_result.append_diff_op("HelloWorld", &DiffOp::AddStateTag);
        expected_diff_result.append_diff_op("HelloWorld", &DiffOp::CreateSchedule);

        similar_asserts::assert_eq!(expected_diff_result, actual_diff_result);
    }
}
