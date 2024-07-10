use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CloudWatchLogsLogGroup {
    pub log_group_arn: Option<String>,
}

impl From<aws_sdk_sfn::types::CloudWatchLogsLogGroup> for CloudWatchLogsLogGroup {
    fn from(value: aws_sdk_sfn::types::CloudWatchLogsLogGroup) -> Self {
        CloudWatchLogsLogGroup {
            log_group_arn: value.log_group_arn().map(|lga| lga.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LogDestination {
    pub cloud_watch_logs_log_group: Option<CloudWatchLogsLogGroup>,
}

impl From<aws_sdk_sfn::types::LogDestination> for LogDestination {
    fn from(value: aws_sdk_sfn::types::LogDestination) -> Self {
        LogDestination {
            cloud_watch_logs_log_group: value
                .cloud_watch_logs_log_group()
                .map(|cwlg| CloudWatchLogsLogGroup::from(cwlg.clone())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoggingConfiguration {
    pub level: Option<String>,
    pub include_execution_data: bool,
    pub destinations: Vec<LogDestination>,
}

impl From<aws_sdk_sfn::types::LoggingConfiguration> for LoggingConfiguration {
    fn from(value: aws_sdk_sfn::types::LoggingConfiguration) -> Self {
        LoggingConfiguration {
            destinations: value
                .destinations()
                .iter()
                .map(|d| LogDestination::from(d.clone()))
                .collect(),
            include_execution_data: value.include_execution_data(),
            level: value.level().map(|l| l.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TracingConfiguration {
    pub enabled: bool,
}

impl From<aws_sdk_sfn::types::TracingConfiguration> for TracingConfiguration {
    fn from(value: aws_sdk_sfn::types::TracingConfiguration) -> Self {
        TracingConfiguration {
            enabled: value.enabled(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StateMachine {
    pub name: String,
    pub status: Option<String>,
    pub definition: String,
    pub role_arn: String,
    pub r#type: String,
    pub logging_configuration: Option<LoggingConfiguration>,
    pub tracing_configuration: Option<TracingConfiguration>,
    pub label: Option<String>,
    pub revision_id: Option<String>,
    pub description: Option<String>,
}

impl From<aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineOutput>
    for StateMachine
{
    fn from(
        value: aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineOutput,
    ) -> Self {
        StateMachine {
            name: value.name().to_string(),
            status: value.status().map(|s| s.to_string()),
            definition: value.definition().to_string(),
            role_arn: value.role_arn().to_string(),
            r#type: value.r#type().to_string(),
            logging_configuration: value
                .logging_configuration()
                .map(|lc| LoggingConfiguration::from(lc.clone())),
            tracing_configuration: value
                .tracing_configuration()
                .map(|tc| TracingConfiguration::from(tc.clone())),
            label: value.label().map(|l| l.to_string()),
            revision_id: value.revision_id().map(|ri| ri.to_string()),
            description: value.description().map(|d| d.to_string()),
        }
    }
}
