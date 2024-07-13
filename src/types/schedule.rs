use serde::{Deserialize, Serialize};

use super::ResourceTag;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexibleTimeWindow {
    pub mode: String,
    pub maximum_window_in_minutes: Option<i32>,
}

impl From<aws_sdk_scheduler::types::FlexibleTimeWindow> for FlexibleTimeWindow {
    fn from(value: aws_sdk_scheduler::types::FlexibleTimeWindow) -> Self {
        FlexibleTimeWindow {
            mode: value.mode().to_string(),
            maximum_window_in_minutes: value.maximum_window_in_minutes(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeadLetterConfig {
    pub arn: Option<String>,
}

impl From<aws_sdk_scheduler::types::DeadLetterConfig> for DeadLetterConfig {
    fn from(value: aws_sdk_scheduler::types::DeadLetterConfig) -> Self {
        DeadLetterConfig { arn: value.arn }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CapacityProviderStrategyItem {
    pub base: i32,
    pub capacity_provider: String,
    pub weight: i32,
}

impl From<aws_sdk_scheduler::types::CapacityProviderStrategyItem> for CapacityProviderStrategyItem {
    fn from(value: aws_sdk_scheduler::types::CapacityProviderStrategyItem) -> Self {
        CapacityProviderStrategyItem {
            base: value.base(),
            capacity_provider: value.capacity_provider().to_string(),
            weight: value.weight(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AwsVpcConfiguration {
    pub assign_public_ip: Option<String>,
    pub security_groups: Vec<String>,
    pub subnets: Vec<String>,
}

impl From<aws_sdk_scheduler::types::AwsVpcConfiguration> for AwsVpcConfiguration {
    fn from(value: aws_sdk_scheduler::types::AwsVpcConfiguration) -> Self {
        AwsVpcConfiguration {
            assign_public_ip: value.assign_public_ip().map(|s| s.to_string()),
            security_groups: value.security_groups().to_vec(),
            subnets: value.subnets().to_vec(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfiguration {
    pub awsvpc_configuration: AwsVpcConfiguration,
}

impl From<aws_sdk_scheduler::types::NetworkConfiguration> for NetworkConfiguration {
    fn from(value: aws_sdk_scheduler::types::NetworkConfiguration) -> Self {
        // XXX: Could we avoid clone here?
        let awsvpc_configuration = value.awsvpc_configuration().unwrap().clone();
        NetworkConfiguration {
            awsvpc_configuration: AwsVpcConfiguration::from(awsvpc_configuration),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlacementConstraint {
    pub expression: Option<String>,
    pub r#type: Option<String>,
}

impl From<aws_sdk_scheduler::types::PlacementConstraint> for PlacementConstraint {
    fn from(value: aws_sdk_scheduler::types::PlacementConstraint) -> Self {
        PlacementConstraint {
            expression: value.expression().map(|s| s.to_string()),
            r#type: value.r#type().map(|s| s.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlacementStrategy {
    pub field: Option<String>,
    pub r#type: Option<String>,
}

impl From<aws_sdk_scheduler::types::PlacementStrategy> for PlacementStrategy {
    fn from(value: aws_sdk_scheduler::types::PlacementStrategy) -> Self {
        PlacementStrategy {
            field: value.field().map(|s| s.to_string()),
            r#type: value.r#type().map(|s| s.to_string()),
        }
    }
}

type PlacementConstraintList = Vec<PlacementConstraint>;
type PlacementStrategyList = Vec<PlacementStrategy>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EcsParameters {
    pub task_definition_arn: String,
    pub capacity_provider_strategy: Vec<CapacityProviderStrategyItem>,
    pub enable_ecs_managed_tags: Option<bool>,
    pub enable_execute_command: Option<bool>,
    pub group: Option<String>,
    pub launch_type: Option<String>,
    pub network_configuration: Option<NetworkConfiguration>,
    pub placement_constraints: PlacementConstraintList,
    pub placement_strategy: PlacementStrategyList,
    pub platform_version: Option<String>,
    pub propagate_tags: Option<String>,
    pub reference_id: Option<String>,
    pub tags: Vec<ResourceTag>,
    pub task_count: Option<i32>,
}

impl From<aws_sdk_scheduler::types::EcsParameters> for EcsParameters {
    fn from(value: aws_sdk_scheduler::types::EcsParameters) -> Self {
        let capacity_provider_strategy = value.capacity_provider_strategy();
        let network_configuration = value.network_configuration();

        EcsParameters {
            task_definition_arn: value.task_definition_arn().to_string(),
            capacity_provider_strategy: capacity_provider_strategy
                .iter()
                .map(|cp| CapacityProviderStrategyItem::from(cp.clone()))
                .collect(),
            enable_ecs_managed_tags: value.enable_ecs_managed_tags(),
            enable_execute_command: value.enable_execute_command(),
            group: value.group().map(|s| s.to_string()),
            launch_type: value.launch_type().map(|s| s.to_string()),
            network_configuration: network_configuration
                .map(|nc| NetworkConfiguration::from(nc.clone())),
            placement_constraints: value
                .placement_constraints()
                .iter()
                .map(|pc| PlacementConstraint::from(pc.clone()))
                .collect(),
            placement_strategy: value
                .placement_strategy()
                .iter()
                .map(|ps| PlacementStrategy::from(ps.clone()))
                .collect(),
            platform_version: value.platform_version().map(|s| s.to_string()),
            propagate_tags: value.propagate_tags().map(|s| s.to_string()),
            reference_id: value.reference_id().map(|s| s.to_string()),
            tags: value
                .tags()
                .iter()
                .map(|kv| ResourceTag::from(kv.clone()))
                .collect(),
            task_count: value.task_count(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KinesisParameters {
    pub partition_key: String,
}

impl From<aws_sdk_scheduler::types::KinesisParameters> for KinesisParameters {
    fn from(value: aws_sdk_scheduler::types::KinesisParameters) -> Self {
        KinesisParameters {
            partition_key: value.partition_key().to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    pub maximum_event_age_in_seconds: Option<i32>,
    pub maximum_retry_attempts: Option<i32>,
}

impl From<aws_sdk_scheduler::types::RetryPolicy> for RetryPolicy {
    fn from(value: aws_sdk_scheduler::types::RetryPolicy) -> Self {
        RetryPolicy {
            maximum_event_age_in_seconds: value.maximum_event_age_in_seconds(),
            maximum_retry_attempts: value.maximum_retry_attempts(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SageMakerPipelineParameter {
    pub name: String,
    pub value: String,
}

impl From<aws_sdk_scheduler::types::SageMakerPipelineParameter> for SageMakerPipelineParameter {
    fn from(value: aws_sdk_scheduler::types::SageMakerPipelineParameter) -> Self {
        SageMakerPipelineParameter {
            name: value.name().to_string(),
            value: value.value().to_string(),
        }
    }
}

type SageMakerPipelineParameterList = Vec<SageMakerPipelineParameter>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SageMakerPipelineParameters {
    pub pipeline_parameter_list: SageMakerPipelineParameterList,
}

impl From<aws_sdk_scheduler::types::SageMakerPipelineParameters> for SageMakerPipelineParameters {
    fn from(value: aws_sdk_scheduler::types::SageMakerPipelineParameters) -> Self {
        let pipeline_parameter_list = value.pipeline_parameter_list();

        SageMakerPipelineParameters {
            pipeline_parameter_list: pipeline_parameter_list
                .iter()
                .map(|pp| SageMakerPipelineParameter::from(pp.clone()))
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SqsParameters {
    pub message_group_id: Option<String>,
}

impl From<aws_sdk_scheduler::types::SqsParameters> for SqsParameters {
    fn from(value: aws_sdk_scheduler::types::SqsParameters) -> Self {
        SqsParameters {
            message_group_id: value.message_group_id().map(|s| s.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventBridgeParameters {
    pub detail_type: String,
    pub source: String,
}

impl From<aws_sdk_scheduler::types::EventBridgeParameters> for EventBridgeParameters {
    fn from(value: aws_sdk_scheduler::types::EventBridgeParameters) -> Self {
        EventBridgeParameters {
            detail_type: value.detail_type().to_string(),
            source: value.source().to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleTarget {
    pub arn: String,
    pub role_arn: String,
    pub dead_letter_config: Option<DeadLetterConfig>,
    pub ecs_parameters: Option<EcsParameters>,
    pub event_bridge_parameters: Option<EventBridgeParameters>,
    pub input: Option<String>,
    pub kinesis_parameters: Option<KinesisParameters>,
    pub retry_policy: Option<RetryPolicy>,
    pub sage_maker_pipeline_parameters: Option<SageMakerPipelineParameters>,
    pub sqs_parameters: Option<SqsParameters>,
}

impl From<aws_sdk_scheduler::types::Target> for ScheduleTarget {
    fn from(value: aws_sdk_scheduler::types::Target) -> Self {
        let dead_letter_config = value.dead_letter_config();
        let ecs_parameters = value.ecs_parameters();
        let event_bridge_parameters = value.event_bridge_parameters();
        let kinesis_parameters = value.kinesis_parameters();
        let retry_policy = value.retry_policy();
        let sage_maker_pipeline_parameters = value.sage_maker_pipeline_parameters();
        let sqs_parameters = value.sqs_parameters();

        ScheduleTarget {
            arn: value.arn().to_string(),
            role_arn: value.role_arn().to_string(),
            dead_letter_config: dead_letter_config.map(|dlc| DeadLetterConfig::from(dlc.clone())),
            ecs_parameters: ecs_parameters.map(|ecs| EcsParameters::from(ecs.clone())),
            event_bridge_parameters: event_bridge_parameters
                .map(|ebp| EventBridgeParameters::from(ebp.clone())),
            input: value.input().map(|s| s.to_string()),
            kinesis_parameters: kinesis_parameters.map(|kp| KinesisParameters::from(kp.clone())),
            retry_policy: retry_policy.map(|rp| RetryPolicy::from(rp.clone())),
            sage_maker_pipeline_parameters: sage_maker_pipeline_parameters
                .map(|smp| SageMakerPipelineParameters::from(smp.clone())),
            sqs_parameters: sqs_parameters.map(|sp| SqsParameters::from(sp.clone())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub name: String,
    pub description: Option<String>,
    pub end_date: Option<String>,
    pub flexible_time_window: Option<FlexibleTimeWindow>,
    pub group_name: Option<String>,
    pub kms_key_arn: Option<String>,
    pub schedule_expression: String,
    pub schedule_expression_timezone: Option<String>,
    pub start_date: Option<String>,
    pub target: ScheduleTarget,
}

impl Schedule {
    pub fn schedule_name_with_group(&self) -> String {
        match &self.group_name {
            Some(group_name) => format!("{}/{}", group_name, self.name),
            None => panic!("group name should be provided"),
        }
    }
}

impl From<aws_sdk_scheduler::operation::get_schedule::GetScheduleOutput> for Schedule {
    fn from(value: aws_sdk_scheduler::operation::get_schedule::GetScheduleOutput) -> Self {
        let flexible_time_window = value.flexible_time_window();
        let target = value.target().unwrap();

        Schedule {
            name: value.name().unwrap().to_string(),
            description: value.description().map(|s| s.to_string()),
            end_date: value.end_date().map(|s| s.to_string()),
            flexible_time_window: flexible_time_window
                .map(|ftw| FlexibleTimeWindow::from(ftw.clone())),
            group_name: value.group_name().map(|s| s.to_string()),
            kms_key_arn: value.kms_key_arn().map(|s| s.to_string()),
            schedule_expression: value.schedule_expression().unwrap().to_string(),
            schedule_expression_timezone: value
                .schedule_expression_timezone()
                .map(|s| s.to_string()),
            start_date: value.start_date().map(|s| s.to_string()),
            target: ScheduleTarget::from(target.clone()),
        }
    }
}
