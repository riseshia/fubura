use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct FlexibleTimeWindow {
    pub mode: String,
    pub maximum_window_in_minute: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeadLetterConfig {
    pub arn: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CapacityProviderStrategyItem {
    pub base: usize,
    pub capacity_provider: String,
    pub weight: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AwsVpcConfiguration {
    pub assign_public_ip: String,
    pub security_groups: Vec<String>,
    pub subnets: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkConfiguration {
    pub awsvpc_configuration: AwsVpcConfiguration,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlacementConstraint {
    pub expression: String,
    pub r#type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlacementStrategy {
    pub field: String,
    pub r#type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScheduleTag {
    pub key: String,
    pub value: String,
}

type PlacementConstraintList = Vec<PlacementConstraint>;
type PlacementStrategyList = Vec<PlacementStrategy>;
type ScheduleTagList = Vec<ScheduleTag>;

#[derive(Deserialize, Serialize, Debug)]
pub struct EcsParameters {
    pub task_definition_arn: String,
    pub capacity_provider_strategy: Option<Vec<CapacityProviderStrategyItem>>,
    pub enable_ecs_managed_tags: Option<bool>,
    pub enable_execute_command: Option<bool>,
    pub group: Option<String>,
    pub launch_type: Option<String>,
    pub network_configuration: Option<NetworkConfiguration>,
    pub placement_constraints: Option<PlacementConstraintList>,
    pub placement_strategy: Option<PlacementStrategyList>,
    pub platform_version: Option<String>,
    pub propagate_tags: Option<String>,
    pub reference_id: Option<String>,
    pub tags: Option<ScheduleTagList>,
    pub task_count: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KinesisParameters {
    pub partition_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RetryPolicy {
    pub maximum_event_age_in_seconds: Option<usize>,
    pub maximum_retry_attempts: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PipelineParameter {
    pub name: String,
    pub value: String,
}

type SageMakerPipelineParameterList = Vec<PipelineParameter>;

#[derive(Deserialize, Serialize, Debug)]
pub struct SageMakerPipelineParameters {
    pub pipeline_parameter_list: Option<SageMakerPipelineParameterList>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SqsParameters {
    pub message_group_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventBridgeParameters {
    pub detail_type: String,
    pub source: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Target {
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Schedule {
    pub name: String,
    pub description: Option<String>,
    pub end_date: Option<String>,
    pub flexible_time_window: FlexibleTimeWindow,
    pub group_name: Option<String>,
    pub kms_key_arn: Option<String>,
    pub schedule_expression: String,
    pub schedule_expression_timezone: Option<String>,
    pub start_date: Option<String>,
    pub target: Target,
}
