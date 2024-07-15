use crate::context::Context;
use crate::types::{Config, SsConfig};
use crate::{scheduler, sfn, sts};

pub struct ImportCommand;

impl ImportCommand {
    pub async fn run(
        context: &Context,
        config_path: &str,
        mut config: Config,
        sfn_name: &str,
        schedule_name_with_group: &Option<String>,
    ) {
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
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use aws_sdk_scheduler::{
        operation::get_schedule::builders::GetScheduleOutputBuilder, types::builders::TargetBuilder,
    };
    use aws_sdk_sfn::operation::list_tags_for_resource::builders::ListTagsForResourceOutputBuilder;
    use aws_sdk_sfn::types::builders::TagBuilder;
    use aws_sdk_sfn::{
        operation::describe_state_machine::builders::DescribeStateMachineOutputBuilder,
        primitives::{DateTime, DateTimeFormat},
        types::StateMachineType,
    };
    use aws_sdk_sts::operation::get_caller_identity::builders::GetCallerIdentityOutputBuilder;

    use mockall::predicate::eq;

    use serde_json::Value;

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
        .await;

        let config =
            std::fs::read_to_string(imported_config_path).expect("imported config not found");
        let v: Value = serde_json::from_str(&config).expect("imported config is not valid json");

        similar_asserts::assert_eq!(
            v,
            serde_json::json!({
                "ssConfigs": [{
                    "state": {
                        "name": "HelloWorld",
                        "type": "STANDARD",
                        "definition": {
                            "StartAt": "FirstState"
                        },
                        "description": null,
                        "label": null,
                        "loggingConfiguration": null,
                        "tracingConfiguration": null,
                        "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                        "revisionId": null,
                        "status": null,
                        "tags": [
                            {
                                "key": "Name",
                                "value": "HelloWorld"
                            }
                        ]
                    },
                    "schedule": {
                        "groupName": "default",
                        "name": "HelloWorld",
                        "description": "HellowWorld schedule",
                        "endDate": null,
                        "startDate": null,
                        "flexibleTimeWindow": null,
                        "kmsKeyArn": null,
                        "scheduleExpression": "rate(1 minute)",
                        "scheduleExpressionTimezone": "UTC",
                        "state": "ENABLED",
                        "target": {
                            "arn": "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
                            "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                            "deadLetterConfig": null,
                            "ecsParameters": null,
                            "eventBridgeParameters": null,
                            "input": null,
                            "kinesisParameters": null,
                            "retryPolicy": null,
                            "sageMakerPipelineParameters": null,
                            "sqsParameters": null,
                        },
                    },
                    "deleteAll": false,
                    "deleteSchedule": false,
                }
            ]})
        );
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
        .await;

        let config =
            std::fs::read_to_string(imported_config_path).expect("imported config not found");
        let v: Value = serde_json::from_str(&config).expect("imported config is not valid json");

        similar_asserts::assert_eq!(
            v,
            serde_json::json!({
                "ssConfigs": [{
                    "state": {
                        "name": "HelloWorld",
                        "type": "STANDARD",
                        "definition": {
                            "StartAt": "FirstState"
                        },
                        "description": null,
                        "label": null,
                        "loggingConfiguration": null,
                        "tracingConfiguration": null,
                        "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                        "revisionId": null,
                        "status": null,
                        "tags": [
                            {
                                "key": "Name",
                                "value": "HelloWorld"
                            }
                        ]
                    },
                    "schedule": null,
                    "deleteAll": false,
                    "deleteSchedule": false,
                }
            ]})
        );
    }
}
