use std::process::exit;

use crate::context::Context;
use crate::{scheduler, sfn};

pub struct ExportCommand;

impl ExportCommand {
    pub async fn run(
        context: &Context,
        config: &str,
        sfn_arn: &str,
        schedule_name_with_group: &Option<String>,
    ) {
        let res = sfn::describe_state_machine(&context.sfn_client, sfn_arn)
            .await
            .unwrap_or_else(|err| {
                eprintln!("failed to describe state machine: {}", err);
                exit(1);
            });

        let sfn_config = serde_json::json!({
            "name": res.name(),
            "type": res.r#type().to_string(),
            "definition": res.definition(),
            "roleArn": res.role_arn(),
            "creationDate": res.creation_date().to_string(),
        });

        let scheduler_config = if let Some(schedule_name_with_group) = schedule_name_with_group {
            let (group_name, schedule_name) =
                schedule_name_with_group.split_once('/').unwrap_or_else(|| {
                    eprintln!(
                        "invalid schedule name with group: {:?}",
                        schedule_name_with_group
                    );
                    exit(1);
                });

            let res = scheduler::get_schedule(&context.scheduler_client, group_name, schedule_name)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("failed to get schedule: {}", err);
                    exit(1);
                });

            Some(serde_json::json!({
                "arn": res.arn(),
                "groupName": res.group_name(),
                "name": res.name(),
                "description": res.description(),
                "scheduleExpression": res.schedule_expression(),
                "scheduleExpressionTimezone": res.schedule_expression_timezone(),
                "target": {
                    "arn": res.target().unwrap().arn(),
                    "roleArn": res.target().unwrap().role_arn(),
                },
            }))
        } else {
            None
        };

        let full_config = serde_json::json!({
            "schedule": scheduler_config,
            "state": sfn_config,
        });

        std::fs::write(config, serde_json::to_string_pretty(&full_config).unwrap())
            .expect("failed to write config to file");

        println!("export called with");
        println!("- export path: {}", config);
        println!("- target sfn arn: {}", sfn_arn);
        println!("- target schedule arn: {:?}", schedule_name_with_group);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use aws_sdk_scheduler::{
        operation::get_schedule::builders::GetScheduleOutputBuilder, types::builders::TargetBuilder,
    };
    use aws_sdk_sfn::{
        operation::describe_state_machine::builders::DescribeStateMachineOutputBuilder,
        primitives::DateTime, primitives::DateTimeFormat, types::StateMachineType,
    };
    use mockall::predicate::eq;
    use serde_json::Value;

    #[tokio::test]
    async fn test_sfn_arn_schedule_arn_given() {
        let mut context = Context::async_default().await;

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
                    .definition("...".to_string())
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
                    .target(
                        TargetBuilder::default()
                            .arn("arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld")
                            .role_arn("arn:aws:iam::123456789012:role/service-role/HelloWorldRole")
                            .build()
                            .unwrap(),
                    )
                    .build())
            });

        let exported_config_path = "tmp/hello-world.jsonnet";
        std::fs::remove_file(exported_config_path).ok();

        ExportCommand::run(
            &context,
            exported_config_path,
            "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            &Some("default/HelloWorld".to_string()),
        )
        .await;

        let config =
            std::fs::read_to_string(exported_config_path).expect("exported config not found");
        let v: Value = serde_json::from_str(&config).expect("exported config is not valid json");

        assert_eq!(
            v,
            serde_json::json!({
                "state": {
                    "name": "HelloWorld",
                    "type": "STANDARD",
                    "definition": "...",
                    "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                    "creationDate": "2021-01-01T00:00:00Z",
                },
                "schedule": {
                    "arn": "arn:aws:scheduler:us-west-2:123456789012:schedule:default/HelloWorld",
                    "groupName": "default",
                    "name": "HelloWorld",
                    "description": "HellowWorld schedule",
                    "scheduleExpression": "rate(1 minute)",
                    "scheduleExpressionTimezone": "UTC",
                    "target": {
                        "arn": "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
                        "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                    },
                }
            })
        );
    }

    #[tokio::test]
    async fn test_sfn_arn_given() {
        let mut context = Context::async_default().await;

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
                    .definition("...".to_string())
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

        let exported_config_path = "tmp/hello-world-without-schedule.jsonnet";
        std::fs::remove_file(exported_config_path).ok();

        ExportCommand::run(
            &context,
            exported_config_path,
            "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
            &None,
        )
        .await;

        let config =
            std::fs::read_to_string(exported_config_path).expect("exported config not found");
        let v: Value = serde_json::from_str(&config).expect("exported config is not valid json");

        assert_eq!(
            v,
            serde_json::json!({
                "state": {
                    "name": "HelloWorld",
                    "type": "STANDARD",
                    "definition": "...",
                    "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                    "creationDate": "2021-01-01T00:00:00Z",
                },
                "schedule": null,
            })
        );
    }
}
