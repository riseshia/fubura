use anyhow::Result;

use crate::context::Context;
use crate::types::{Config, SsConfig};
use crate::{scheduler, sfn, sts};

pub struct ImportCommand;

fn ensure_not_exist_in_config(config: &Config, sfn_name: &str) {
    if config
        .ss_configs
        .iter()
        .any(|ss_config| ss_config.state.name == sfn_name)
    {
        panic!("state machine '{}' already exists in config", sfn_name);
    }
}

impl ImportCommand {
    pub async fn run(
        context: &Context,
        config_path: &str,
        mut config: Config,
        sfn_name: &str,
        schedule_name_with_group: &Option<String>,
    ) -> Result<()> {
        ensure_not_exist_in_config(&config, sfn_name);

        let state_arn_prefix = sts::build_state_arn_prefix(context).await;
        let state_arn = format!("{}{}", state_arn_prefix, sfn_name);

        let state_machine = sfn::describe_state_machine_with_tags(&context.sfn_client, &state_arn)
            .await
            .unwrap_or_else(|| panic!("state machine not found: {}", state_arn));

        let scheduler_config = if let Some(schedule_name_with_group) = schedule_name_with_group {
            let schedule =
                scheduler::get_schedule(&context.scheduler_client, schedule_name_with_group)
                    .await
                    .unwrap_or_else(|| panic!("schedule not found: {}", state_arn));

            Some(schedule)
        } else {
            None
        };

        let ss_config = SsConfig {
            state: state_machine,
            schedule: scheduler_config,
            delete_all: false,
            delete_schedule: false,
        };

        config.ss_configs.push(ss_config);

        std::fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap_or_else(
            |e| {
                panic!(
                    "failed to write config to file '{}' with error: {}",
                    config_path, e
                )
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::types::{Schedule, StateMachine};

    use super::*;

    use aws_sdk_scheduler::{
        operation::get_schedule::builders::GetScheduleOutputBuilder, types::builders::TargetBuilder,
    };
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

    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_sfn_name_schedule_name_given() {
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

        let imported_config_path = "tmp/hello-world.jsonnet";
        std::fs::remove_file(imported_config_path).ok();

        ImportCommand::run(
            &context,
            imported_config_path,
            Config::default(),
            "HelloWorld",
            &Some("default/HelloWorld".to_string()),
        )
        .await
        .unwrap();

        let config_str =
            std::fs::read_to_string(imported_config_path).expect("imported config not found");
        let actual_config: Config =
            serde_json::from_str(&config_str).expect("imported config is not valid json");

        let expected_config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: Some(Schedule::test_default()),
                delete_all: false,
                delete_schedule: false,
            }],
        };

        similar_asserts::assert_eq!(actual_config, expected_config);
    }

    #[tokio::test]
    async fn test_sfn_name_given() {
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
                    .creation_date(
                        DateTime::from_str("2021-01-01T00:00:00Z", DateTimeFormat::DateTime)
                            .unwrap(),
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

        let imported_config_path = "tmp/hello-world-without-schedule.jsonnet";
        std::fs::remove_file(imported_config_path).ok();

        ImportCommand::run(
            &context,
            imported_config_path,
            Config::default(),
            "HelloWorld",
            &None,
        )
        .await
        .unwrap();

        let config_str =
            std::fs::read_to_string(imported_config_path).expect("imported config not found");
        let actual_config: Config =
            serde_json::from_str(&config_str).expect("imported config is not valid");
        let expected_config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: None,
                delete_all: false,
                delete_schedule: false,
            }],
        };

        similar_asserts::assert_eq!(actual_config, expected_config);
    }

    #[tokio::test]
    #[should_panic(expected = "state machine 'HelloWorld' already exists in config")]
    async fn test_import_fail_with_already_exists() {
        let context = Context::async_default().await;

        let imported_config_path = "tmp/test-import-fail-with-already-exists.jsonnet";
        std::fs::remove_file(imported_config_path).ok();

        let config = Config {
            ss_configs: vec![SsConfig {
                state: StateMachine::test_default(),
                schedule: Some(Schedule::test_default()),
                delete_all: false,
                delete_schedule: false,
            }],
        };
        std::fs::write(
            imported_config_path,
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        ImportCommand::run(&context, imported_config_path, config, "HelloWorld", &None)
            .await
            .unwrap();
    }
}
