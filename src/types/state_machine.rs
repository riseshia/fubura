use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ResourceTag;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
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

impl From<CloudWatchLogsLogGroup> for aws_sdk_sfn::types::CloudWatchLogsLogGroup {
    fn from(value: CloudWatchLogsLogGroup) -> Self {
        let log_group_arn = value
            .log_group_arn
            .unwrap_or_else(|| panic!("log_group_arn is required for CloudWatchLogsLogGroup"));

        aws_sdk_sfn::types::builders::CloudWatchLogsLogGroupBuilder::default()
            .log_group_arn(log_group_arn)
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
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

impl From<LogDestination> for aws_sdk_sfn::types::LogDestination {
    fn from(value: LogDestination) -> Self {
        let mut builder = aws_sdk_sfn::types::builders::LogDestinationBuilder::default();

        if let Some(cloud_watch_logs_log_group) = value.cloud_watch_logs_log_group {
            builder = builder.cloud_watch_logs_log_group(cloud_watch_logs_log_group.into())
        }

        builder.build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum LogLevel {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "INFO")]
    Fatal,
    #[serde(rename = "OFF")]
    Off,
}

impl From<aws_sdk_sfn::types::LogLevel> for LogLevel {
    fn from(value: aws_sdk_sfn::types::LogLevel) -> Self {
        match value {
            aws_sdk_sfn::types::LogLevel::All => LogLevel::All,
            aws_sdk_sfn::types::LogLevel::Error => LogLevel::Error,
            aws_sdk_sfn::types::LogLevel::Fatal => LogLevel::Fatal,
            aws_sdk_sfn::types::LogLevel::Off => LogLevel::Off,
            _ => panic!("unknown log level: {:?}", value),
        }
    }
}

impl From<LogLevel> for aws_sdk_sfn::types::LogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::All => aws_sdk_sfn::types::LogLevel::All,
            LogLevel::Error => aws_sdk_sfn::types::LogLevel::Error,
            LogLevel::Fatal => aws_sdk_sfn::types::LogLevel::Fatal,
            LogLevel::Off => aws_sdk_sfn::types::LogLevel::Off,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfiguration {
    pub level: Option<LogLevel>,
    #[serde(rename = "includeExecutionData")]
    pub include_execution_data: Option<bool>,
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
            include_execution_data: Some(value.include_execution_data()),
            level: value.level().map(|l| LogLevel::from(l.clone())),
        }
    }
}

impl From<LoggingConfiguration> for aws_sdk_sfn::types::LoggingConfiguration {
    fn from(value: LoggingConfiguration) -> Self {
        let mut builder = aws_sdk_sfn::types::builders::LoggingConfigurationBuilder::default();

        if value.destinations.len() > 1 {
            panic!("destinations size is limited to 1.");
        }

        if let Some(destination) = value.destinations.first() {
            builder = builder.destinations(destination.clone().into());
        }

        if let Some(level) = value.level {
            builder = builder.level(level.into());
        }

        if let Some(include_execution_data) = value.include_execution_data {
            builder = builder.include_execution_data(include_execution_data);
        }

        builder.build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
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

impl From<TracingConfiguration> for aws_sdk_sfn::types::TracingConfiguration {
    fn from(value: TracingConfiguration) -> Self {
        aws_sdk_sfn::types::builders::TracingConfigurationBuilder::default()
            .enabled(value.enabled)
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum StateMachineType {
    #[serde(rename = "STANDARD")]
    Standard,
    #[serde(rename = "EXPRESS")]
    Express,
}

impl From<aws_sdk_sfn::types::StateMachineType> for StateMachineType {
    fn from(value: aws_sdk_sfn::types::StateMachineType) -> Self {
        match value {
            aws_sdk_sfn::types::StateMachineType::Standard => StateMachineType::Standard,
            aws_sdk_sfn::types::StateMachineType::Express => StateMachineType::Express,
            _ => panic!("unknown state machine type: {:?}", value),
        }
    }
}

impl From<StateMachineType> for aws_sdk_sfn::types::StateMachineType {
    fn from(value: StateMachineType) -> Self {
        match value {
            StateMachineType::Standard => aws_sdk_sfn::types::StateMachineType::Standard,
            StateMachineType::Express => aws_sdk_sfn::types::StateMachineType::Express,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateMachine {
    pub name: String,
    pub definition: Value,
    pub role_arn: String,
    pub r#type: StateMachineType,
    pub logging_configuration: Option<LoggingConfiguration>,
    pub tracing_configuration: Option<TracingConfiguration>,
    pub tags: Vec<ResourceTag>,
}

impl StateMachine {
    #[cfg(test)]
    pub fn test_default() -> Self {
        StateMachine {
            name: "HelloWorld".to_string(),
            r#type: StateMachineType::Standard,
            definition: serde_json::json!({
                "StartAt": "FirstState",
            }),
            role_arn: "arn:aws:iam::123456789012:role/service-role/HelloWorldRole".to_string(),
            logging_configuration: Some(LoggingConfiguration {
                level: Some(LogLevel::All),
                include_execution_data: Some(true),
                destinations: vec![LogDestination {
                    cloud_watch_logs_log_group: Some(CloudWatchLogsLogGroup {
                        log_group_arn: Some(
                            "arn:aws:logs:us-west-2:123456789012:log-group:HelloWorldLogGroup"
                                .to_string(),
                        ),
                    }),
                }],
            }),
            tracing_configuration: None,
            tags: vec![ResourceTag {
                key: "Name".to_string(),
                value: "HelloWorld".to_string(),
            }],
        }
    }
}

impl From<aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineOutput>
    for StateMachine
{
    fn from(
        value: aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineOutput,
    ) -> Self {
        StateMachine {
            name: value.name().to_string(),
            definition: serde_json::from_str(value.definition()).unwrap(),
            role_arn: value.role_arn().to_string(),
            r#type: StateMachineType::from(value.r#type().clone()),
            logging_configuration: value
                .logging_configuration()
                .map(|lc| LoggingConfiguration::from(lc.clone())),
            tracing_configuration: value
                .tracing_configuration()
                .map(|tc| TracingConfiguration::from(tc.clone())),
            tags: vec![],
        }
    }
}
