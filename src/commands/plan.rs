use similar::TextDiff;

use crate::context::Context;
use crate::types::SsConfig;
use crate::{scheduler, sfn, sts};

fn print_diff(remote: &str, local: &str) {
    let diff = TextDiff::from_lines(remote, local);

    diff.unified_diff()
        .header("remote", "local")
        .to_writer(std::io::stdout())
        .unwrap();
}

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(context: &Context, config: &SsConfig) {
        let sfn_arn_prefix = sts::build_sfn_arn_prefix(context).await;
        let sfn_arn = format!("{}{}", sfn_arn_prefix, config.state.name);

        let remote_sfn = sfn::describe_state_machine(&context.sfn_client, &sfn_arn).await;

        let remote_schedule = if let Some(schedule_config) = &config.schedule {
            scheduler::get_schedule(
                &context.scheduler_client,
                &schedule_config.schedule_name_with_group(),
            )
            .await
        } else {
            None
        };

        let remote_sfn_json_string = serde_json::to_string_pretty(&remote_sfn).unwrap();
        let local_sfn_json_string = serde_json::to_string_pretty(&config.state).unwrap();

        let mut has_no_diff = true;

        if local_sfn_json_string != remote_sfn_json_string {
            has_no_diff = false;
            print_diff(&remote_sfn_json_string, &local_sfn_json_string);
        }

        if let Some(local_schedule) = &config.schedule {
            let local_schedule_json_string = serde_json::to_string_pretty(&local_schedule).unwrap();

            match remote_schedule {
                Some(remote_schedule) => {
                    let remote_schedule_json_string =
                        serde_json::to_string_pretty(&remote_schedule).unwrap();

                    if remote_schedule_json_string != local_schedule_json_string {
                        has_no_diff = false;
                        print_diff(&remote_schedule_json_string, &local_schedule_json_string);
                    }
                }
                None => {
                    print_diff("null", &local_schedule_json_string);
                }
            }
        } else {
            todo!("no local schedule & remote schedule exists/not exists case");
        }

        if has_no_diff {
            println!("no difference");
        } else {
            std::process::exit(1);
        }
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

        let ss_config_json = serde_json::json!({
            "state": {
                "name": "HelloWorld",
                "type": "STANDARD",
                "definition": "...",
                "description": null,
                "label": null,
                "loggingConfiguration": null,
                "tracingConfiguration": null,
                "roleArn": "arn:aws:iam::123456789012:role/service-role/HelloWorldRole",
                "revisionId": null,
                "status": null,
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
        });
        let ss_config: SsConfig = serde_json::from_value(ss_config_json).unwrap();

        PlanCommand::run(&context, &ss_config).await;
    }
}
