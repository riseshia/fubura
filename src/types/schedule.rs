use aws_sdk_scheduler::primitives::DateTime;
use serde::{Deserialize, Serialize};

use super::ResourceTag;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum FlexibleTimeWindowMode {
    #[serde(rename = "OFF")]
    Off,
    #[serde(rename = "FLEXIBLE")]
    Flexible,
}

impl From<aws_sdk_scheduler::types::FlexibleTimeWindowMode> for FlexibleTimeWindowMode {
    fn from(value: aws_sdk_scheduler::types::FlexibleTimeWindowMode) -> Self {
        match value {
            aws_sdk_scheduler::types::FlexibleTimeWindowMode::Off => FlexibleTimeWindowMode::Off,
            aws_sdk_scheduler::types::FlexibleTimeWindowMode::Flexible => {
                FlexibleTimeWindowMode::Flexible
            }
            _ => panic!("Unexpected flexible time window mode"),
        }
    }
}

impl From<FlexibleTimeWindowMode> for aws_sdk_scheduler::types::FlexibleTimeWindowMode {
    fn from(value: FlexibleTimeWindowMode) -> Self {
        match value {
            FlexibleTimeWindowMode::Off => aws_sdk_scheduler::types::FlexibleTimeWindowMode::Off,
            FlexibleTimeWindowMode::Flexible => {
                aws_sdk_scheduler::types::FlexibleTimeWindowMode::Flexible
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexibleTimeWindow {
    pub mode: FlexibleTimeWindowMode,
    pub maximum_window_in_minutes: Option<i32>,
}

impl From<aws_sdk_scheduler::types::FlexibleTimeWindow> for FlexibleTimeWindow {
    fn from(value: aws_sdk_scheduler::types::FlexibleTimeWindow) -> Self {
        FlexibleTimeWindow {
            mode: FlexibleTimeWindowMode::from(value.mode().clone()),
            maximum_window_in_minutes: value.maximum_window_in_minutes(),
        }
    }
}

impl From<FlexibleTimeWindow> for aws_sdk_scheduler::types::FlexibleTimeWindow {
    fn from(value: FlexibleTimeWindow) -> Self {
        aws_sdk_scheduler::types::builders::FlexibleTimeWindowBuilder::default()
            .mode(value.mode.into())
            .set_maximum_window_in_minutes(value.maximum_window_in_minutes)
            .build()
            .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct DeadLetterConfig {
    pub arn: Option<String>,
}

impl From<aws_sdk_scheduler::types::DeadLetterConfig> for DeadLetterConfig {
    fn from(value: aws_sdk_scheduler::types::DeadLetterConfig) -> Self {
        DeadLetterConfig { arn: value.arn }
    }
}

impl From<DeadLetterConfig> for aws_sdk_scheduler::types::DeadLetterConfig {
    fn from(value: DeadLetterConfig) -> Self {
        aws_sdk_scheduler::types::builders::DeadLetterConfigBuilder::default()
            .set_arn(value.arn)
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<CapacityProviderStrategyItem> for aws_sdk_scheduler::types::CapacityProviderStrategyItem {
    fn from(value: CapacityProviderStrategyItem) -> Self {
        aws_sdk_scheduler::types::builders::CapacityProviderStrategyItemBuilder::default()
            .base(value.base)
            .capacity_provider(value.capacity_provider)
            .weight(value.weight)
            .build()
            .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum AssignPublicIp {
    #[serde(rename = "DISABLED")]
    Disabled,
    #[serde(rename = "ENABLED")]
    Enabled,
}

impl From<aws_sdk_scheduler::types::AssignPublicIp> for AssignPublicIp {
    fn from(value: aws_sdk_scheduler::types::AssignPublicIp) -> Self {
        match value {
            aws_sdk_scheduler::types::AssignPublicIp::Disabled => AssignPublicIp::Disabled,
            aws_sdk_scheduler::types::AssignPublicIp::Enabled => AssignPublicIp::Enabled,
            _ => panic!("Unexpected assign public ip"),
        }
    }
}

impl From<AssignPublicIp> for aws_sdk_scheduler::types::AssignPublicIp {
    fn from(value: AssignPublicIp) -> Self {
        match value {
            AssignPublicIp::Disabled => aws_sdk_scheduler::types::AssignPublicIp::Disabled,
            AssignPublicIp::Enabled => aws_sdk_scheduler::types::AssignPublicIp::Enabled,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AwsVpcConfiguration {
    pub assign_public_ip: Option<AssignPublicIp>,
    pub security_groups: Vec<String>,
    pub subnets: Vec<String>,
}

impl From<aws_sdk_scheduler::types::AwsVpcConfiguration> for AwsVpcConfiguration {
    fn from(value: aws_sdk_scheduler::types::AwsVpcConfiguration) -> Self {
        AwsVpcConfiguration {
            assign_public_ip: value
                .assign_public_ip()
                .map(|s| AssignPublicIp::from(s.clone())),
            security_groups: value.security_groups().to_vec(),
            subnets: value.subnets().to_vec(),
        }
    }
}

impl From<AwsVpcConfiguration> for aws_sdk_scheduler::types::AwsVpcConfiguration {
    fn from(value: AwsVpcConfiguration) -> Self {
        let mut builder = aws_sdk_scheduler::types::builders::AwsVpcConfigurationBuilder::default()
            .set_assign_public_ip(value.assign_public_ip.map(|s| s.into()));

        for sg in value.security_groups {
            builder = builder.security_groups(sg);
        }

        for subnet in value.subnets {
            builder = builder.subnets(subnet);
        }

        builder.build().unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<NetworkConfiguration> for aws_sdk_scheduler::types::NetworkConfiguration {
    fn from(value: NetworkConfiguration) -> Self {
        aws_sdk_scheduler::types::builders::NetworkConfigurationBuilder::default()
            .awsvpc_configuration(value.awsvpc_configuration.into())
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PlacementConstraintType {
    MemberOf,
    DistinctInstance,
}

impl From<aws_sdk_scheduler::types::PlacementConstraintType> for PlacementConstraintType {
    fn from(value: aws_sdk_scheduler::types::PlacementConstraintType) -> Self {
        match value {
            aws_sdk_scheduler::types::PlacementConstraintType::MemberOf => {
                PlacementConstraintType::MemberOf
            }
            aws_sdk_scheduler::types::PlacementConstraintType::DistinctInstance => {
                PlacementConstraintType::DistinctInstance
            }
            _ => panic!("Unexpected placement constraint type"),
        }
    }
}

impl From<PlacementConstraintType> for aws_sdk_scheduler::types::PlacementConstraintType {
    fn from(value: PlacementConstraintType) -> Self {
        match value {
            PlacementConstraintType::MemberOf => {
                aws_sdk_scheduler::types::PlacementConstraintType::MemberOf
            }
            PlacementConstraintType::DistinctInstance => {
                aws_sdk_scheduler::types::PlacementConstraintType::DistinctInstance
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct PlacementConstraint {
    pub expression: Option<String>,
    pub r#type: Option<PlacementConstraintType>,
}

impl From<aws_sdk_scheduler::types::PlacementConstraint> for PlacementConstraint {
    fn from(value: aws_sdk_scheduler::types::PlacementConstraint) -> Self {
        PlacementConstraint {
            expression: value.expression().map(|s| s.to_string()),
            r#type: value
                .r#type()
                .map(|s| PlacementConstraintType::from(s.clone())),
        }
    }
}

impl From<PlacementConstraint> for aws_sdk_scheduler::types::PlacementConstraint {
    fn from(value: PlacementConstraint) -> Self {
        aws_sdk_scheduler::types::builders::PlacementConstraintBuilder::default()
            .set_expression(value.expression)
            .set_type(value.r#type.map(|v| v.into()))
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum PlacementStrategyType {
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "spread")]
    Spread,
    #[serde(rename = "binpack")]
    Binpack,
}

impl From<aws_sdk_scheduler::types::PlacementStrategyType> for PlacementStrategyType {
    fn from(value: aws_sdk_scheduler::types::PlacementStrategyType) -> Self {
        match value {
            aws_sdk_scheduler::types::PlacementStrategyType::Random => {
                PlacementStrategyType::Random
            }
            aws_sdk_scheduler::types::PlacementStrategyType::Spread => {
                PlacementStrategyType::Spread
            }
            aws_sdk_scheduler::types::PlacementStrategyType::Binpack => {
                PlacementStrategyType::Binpack
            }
            _ => panic!("Unexpected placement strategy type"),
        }
    }
}

impl From<PlacementStrategyType> for aws_sdk_scheduler::types::PlacementStrategyType {
    fn from(value: PlacementStrategyType) -> Self {
        match value {
            PlacementStrategyType::Random => {
                aws_sdk_scheduler::types::PlacementStrategyType::Random
            }
            PlacementStrategyType::Spread => {
                aws_sdk_scheduler::types::PlacementStrategyType::Spread
            }
            PlacementStrategyType::Binpack => {
                aws_sdk_scheduler::types::PlacementStrategyType::Binpack
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct PlacementStrategy {
    pub field: Option<String>,
    pub r#type: Option<PlacementStrategyType>,
}

impl From<aws_sdk_scheduler::types::PlacementStrategy> for PlacementStrategy {
    fn from(value: aws_sdk_scheduler::types::PlacementStrategy) -> Self {
        PlacementStrategy {
            field: value.field().map(|s| s.to_string()),
            r#type: value
                .r#type()
                .map(|s| PlacementStrategyType::from(s.clone())),
        }
    }
}

impl From<PlacementStrategy> for aws_sdk_scheduler::types::PlacementStrategy {
    fn from(value: PlacementStrategy) -> Self {
        aws_sdk_scheduler::types::builders::PlacementStrategyBuilder::default()
            .set_field(value.field)
            .set_type(value.r#type.map(|v| v.into()))
            .build()
    }
}

type PlacementConstraintList = Vec<PlacementConstraint>;
type PlacementStrategyList = Vec<PlacementStrategy>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum LaunchType {
    #[serde(rename = "EC2")]
    Ec2,
    #[serde(rename = "FARGATE")]
    Fargate,
    #[serde(rename = "EXTERNAL")]
    External,
}

impl From<aws_sdk_scheduler::types::LaunchType> for LaunchType {
    fn from(value: aws_sdk_scheduler::types::LaunchType) -> Self {
        match value {
            aws_sdk_scheduler::types::LaunchType::Ec2 => LaunchType::Ec2,
            aws_sdk_scheduler::types::LaunchType::Fargate => LaunchType::Fargate,
            aws_sdk_scheduler::types::LaunchType::External => LaunchType::External,
            _ => panic!("Unexpected launch type"),
        }
    }
}

impl From<LaunchType> for aws_sdk_scheduler::types::LaunchType {
    fn from(value: LaunchType) -> Self {
        match value {
            LaunchType::Ec2 => aws_sdk_scheduler::types::LaunchType::Ec2,
            LaunchType::Fargate => aws_sdk_scheduler::types::LaunchType::Fargate,
            LaunchType::External => aws_sdk_scheduler::types::LaunchType::External,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum PropagateTags {
    #[serde(rename = "TASK_DEFINITION")]
    TaskDefinition,
}

impl From<aws_sdk_scheduler::types::PropagateTags> for PropagateTags {
    fn from(value: aws_sdk_scheduler::types::PropagateTags) -> Self {
        match value {
            aws_sdk_scheduler::types::PropagateTags::TaskDefinition => {
                PropagateTags::TaskDefinition
            }
            _ => panic!("Unexpected propagate tags"),
        }
    }
}

impl From<PropagateTags> for aws_sdk_scheduler::types::PropagateTags {
    fn from(value: PropagateTags) -> Self {
        match value {
            PropagateTags::TaskDefinition => {
                aws_sdk_scheduler::types::PropagateTags::TaskDefinition
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EcsParameters {
    pub task_definition_arn: String,
    pub capacity_provider_strategy: Vec<CapacityProviderStrategyItem>,
    pub enable_ecs_managed_tags: Option<bool>,
    pub enable_execute_command: Option<bool>,
    pub group: Option<String>,
    pub launch_type: Option<LaunchType>,
    pub network_configuration: Option<NetworkConfiguration>,
    pub placement_constraints: PlacementConstraintList,
    pub placement_strategy: PlacementStrategyList,
    pub platform_version: Option<String>,
    pub propagate_tags: Option<PropagateTags>,
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
            launch_type: value.launch_type().map(|s| LaunchType::from(s.clone())),
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
            propagate_tags: value
                .propagate_tags()
                .map(|s| PropagateTags::from(s.clone())),
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

impl From<EcsParameters> for aws_sdk_scheduler::types::EcsParameters {
    fn from(value: EcsParameters) -> Self {
        let mut builder = aws_sdk_scheduler::types::builders::EcsParametersBuilder::default()
            .task_definition_arn(value.task_definition_arn)
            .set_enable_ecs_managed_tags(value.enable_ecs_managed_tags)
            .set_enable_execute_command(value.enable_execute_command)
            .set_group(value.group)
            .set_launch_type(value.launch_type.map(|lt| lt.into()))
            .set_network_configuration(value.network_configuration.map(|nc| nc.into()))
            .set_platform_version(value.platform_version)
            .set_propagate_tags(value.propagate_tags.map(|pt| pt.into()))
            .set_reference_id(value.reference_id)
            .set_task_count(value.task_count);

        for capacity_provider_strategy in value.capacity_provider_strategy {
            builder = builder.capacity_provider_strategy(capacity_provider_strategy.into());
        }

        for placement_constraint in value.placement_constraints {
            builder = builder.placement_constraints(placement_constraint.into());
        }

        for placement_strategy in value.placement_strategy {
            builder = builder.placement_strategy(placement_strategy.into());
        }

        for tag in value.tags {
            let mut hash_map = std::collections::HashMap::default();
            hash_map.insert(tag.key, tag.value);
            builder = builder.tags(hash_map);
        }

        builder.build().unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<KinesisParameters> for aws_sdk_scheduler::types::KinesisParameters {
    fn from(value: KinesisParameters) -> Self {
        aws_sdk_scheduler::types::builders::KinesisParametersBuilder::default()
            .partition_key(value.partition_key)
            .build()
            .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<RetryPolicy> for aws_sdk_scheduler::types::RetryPolicy {
    fn from(value: RetryPolicy) -> Self {
        aws_sdk_scheduler::types::builders::RetryPolicyBuilder::default()
            .set_maximum_event_age_in_seconds(value.maximum_event_age_in_seconds)
            .set_maximum_retry_attempts(value.maximum_retry_attempts)
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<SageMakerPipelineParameter> for aws_sdk_scheduler::types::SageMakerPipelineParameter {
    fn from(value: SageMakerPipelineParameter) -> Self {
        aws_sdk_scheduler::types::builders::SageMakerPipelineParameterBuilder::default()
            .name(value.name)
            .value(value.value)
            .build()
            .unwrap()
    }
}

type SageMakerPipelineParameterList = Vec<SageMakerPipelineParameter>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<SageMakerPipelineParameters> for aws_sdk_scheduler::types::SageMakerPipelineParameters {
    fn from(value: SageMakerPipelineParameters) -> Self {
        let mut builder =
            aws_sdk_scheduler::types::builders::SageMakerPipelineParametersBuilder::default();

        for pipeline_parameter in value.pipeline_parameter_list {
            builder = builder.pipeline_parameter_list(pipeline_parameter.into());
        }

        builder.build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<SqsParameters> for aws_sdk_scheduler::types::SqsParameters {
    fn from(value: SqsParameters) -> Self {
        aws_sdk_scheduler::types::builders::SqsParametersBuilder::default()
            .set_message_group_id(value.message_group_id)
            .build()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<EventBridgeParameters> for aws_sdk_scheduler::types::EventBridgeParameters {
    fn from(value: EventBridgeParameters) -> Self {
        aws_sdk_scheduler::types::builders::EventBridgeParametersBuilder::default()
            .detail_type(value.detail_type)
            .source(value.source)
            .build()
            .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
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

impl From<ScheduleTarget> for aws_sdk_scheduler::types::Target {
    fn from(value: ScheduleTarget) -> Self {
        aws_sdk_scheduler::types::builders::TargetBuilder::default()
            .arn(value.arn)
            .role_arn(value.role_arn)
            .set_dead_letter_config(value.dead_letter_config.map(|dlc| dlc.into()))
            .set_ecs_parameters(value.ecs_parameters.map(|ecs| ecs.into()))
            .set_event_bridge_parameters(value.event_bridge_parameters.map(|ebp| ebp.into()))
            .set_input(value.input)
            .set_kinesis_parameters(value.kinesis_parameters.map(|kp| kp.into()))
            .set_retry_policy(value.retry_policy.map(|rp| rp.into()))
            .set_sage_maker_pipeline_parameters(
                value.sage_maker_pipeline_parameters.map(|smp| smp.into()),
            )
            .set_sqs_parameters(value.sqs_parameters.map(|sp| sp.into()))
            .build()
            .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum ScheduleState {
    #[serde(rename = "ENABLED")]
    Enabled,
    #[serde(rename = "DISABLED")]
    Disabled,
}

impl From<aws_sdk_scheduler::types::ScheduleState> for ScheduleState {
    fn from(value: aws_sdk_scheduler::types::ScheduleState) -> Self {
        match value {
            aws_sdk_scheduler::types::ScheduleState::Enabled => ScheduleState::Enabled,
            aws_sdk_scheduler::types::ScheduleState::Disabled => ScheduleState::Disabled,
            _ => panic!("Unexpected schedule state"),
        }
    }
}

impl From<ScheduleState> for aws_sdk_scheduler::types::ScheduleState {
    fn from(value: ScheduleState) -> Self {
        match value {
            ScheduleState::Enabled => aws_sdk_scheduler::types::ScheduleState::Enabled,
            ScheduleState::Disabled => aws_sdk_scheduler::types::ScheduleState::Disabled,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    // AWS handles request with default group if it omitted, so actually we don't need to do this.
    // this default is introduced just for reducing verbose Option unwrap all around.
    #[serde(default = "default_group_name")]
    pub group_name: String,
    pub name: String,
    pub description: Option<String>,
    pub state: ScheduleState,
    pub schedule_expression: String,
    pub schedule_expression_timezone: Option<String>,
    #[serde(with = "datetime_format_as_aws_dt")]
    #[serde(default = "default_date")]
    pub start_date: Option<DateTime>,
    #[serde(with = "datetime_format_as_aws_dt")]
    #[serde(default = "default_date")]
    pub end_date: Option<DateTime>,
    pub flexible_time_window: Option<FlexibleTimeWindow>,
    pub kms_key_arn: Option<String>,
    pub target: ScheduleTarget,
}

impl Schedule {
    pub fn schedule_name_with_group(&self) -> String {
        format!("{}/{}", self.group_name, self.name)
    }

    #[cfg(test)]
    pub fn test_default() -> Self {
        Schedule {
            group_name: "default".to_string(),
            name: "HelloWorld".to_string(),
            description: Some("HellowWorld schedule".to_string()),
            start_date: None,
            end_date: None,
            schedule_expression: "rate(1 minute)".to_string(),
            schedule_expression_timezone: Some("UTC".to_string()),
            flexible_time_window: None,
            kms_key_arn: None,
            state: ScheduleState::Enabled,
            target: ScheduleTarget {
                arn: "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld".to_string(),
                role_arn: "arn:aws:iam::123456789012:role/service-role/HelloWorldRole".to_string(),
                dead_letter_config: None,
                ecs_parameters: None,
                event_bridge_parameters: None,
                input: None,
                kinesis_parameters: None,
                retry_policy: None,
                sage_maker_pipeline_parameters: None,
                sqs_parameters: None,
            },
        }
    }
}

fn default_date() -> Option<DateTime> {
    None
}

fn default_group_name() -> String {
    "default".to_string()
}

mod datetime_format_as_aws_dt {
    use aws_sdk_sts::primitives::{DateTime, DateTimeFormat};
    use serde::Deserialize;

    pub fn serialize<S>(date: &Option<DateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(date) = date {
            let date_str = date.fmt(DateTimeFormat::DateTime).unwrap_or_else(|e| {
                panic!("Fail to parse datetime string {:?}", e);
            });

            serializer.serialize_str(&date_str)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;

        if let Some(s) = s {
            let s = DateTime::from_str(s.as_str(), DateTimeFormat::DateTime).unwrap_or_else(|e| {
                panic!("Fail to parse datetime string {:?}", e);
            });

            Ok(Some(s))
        } else {
            Ok(None)
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
            start_date: value.start_date().copied(),
            end_date: value.end_date().copied(),
            flexible_time_window: flexible_time_window
                .map(|ftw| FlexibleTimeWindow::from(ftw.clone())),
            group_name: value
                .group_name()
                .map_or(default_group_name(), |v| v.to_string()),
            kms_key_arn: value.kms_key_arn().map(|s| s.to_string()),
            schedule_expression: value.schedule_expression().unwrap().to_string(),
            schedule_expression_timezone: value
                .schedule_expression_timezone()
                .map(|s| s.to_string()),
            target: ScheduleTarget::from(target.clone()),
            state: ScheduleState::from(value.state().unwrap().clone()),
        }
    }
}
