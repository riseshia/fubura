use crate::context::Context;
use crate::differ::{build_diff_ops, format_config_diff};
use crate::types::{Config, DiffResult};
use crate::{scheduler, sfn, sts};

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(context: &Context, config: &Config) {
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

            let text_diff =
                format_config_diff(ss_config, &remote_state, &remote_schedule, &diff_ops);
            println!("{}", &text_diff);
            diff_result.append_text_diff(text_diff);
        }

        println!("\nFubura will:");
        for (op, count) in diff_result.summary.iter() {
            println!("    {}: {}", op, count);
        }
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

        let ss_config_json = serde_json::json!({
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
                }
            }]
        });
        let config: Config = serde_json::from_value(ss_config_json).unwrap();

        PlanCommand::run(&context, &config).await;
    }
}
