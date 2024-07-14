use aws_sdk_sfn as sfn;
use aws_sdk_sfn::operation::create_state_machine::{
    CreateStateMachineError, CreateStateMachineOutput,
};
use aws_sdk_sfn::operation::delete_state_machine::{
    DeleteStateMachineError, DeleteStateMachineOutput,
};
use aws_sdk_sfn::operation::describe_state_machine::{
    DescribeStateMachineError, DescribeStateMachineOutput,
};
use aws_sdk_sfn::operation::list_tags_for_resource::{
    ListTagsForResourceError, ListTagsForResourceOutput,
};
use aws_sdk_sfn::operation::tag_resource::{TagResourceError, TagResourceOutput};
use aws_sdk_sfn::operation::untag_resource::{UntagResourceError, UntagResourceOutput};
use aws_sdk_sfn::operation::update_state_machine::{
    UpdateStateMachineError, UpdateStateMachineOutput,
};

#[allow(unused_imports)]
use mockall::automock;

use crate::types::{ResourceTag, StateMachine};

pub struct SfnImpl {
    inner: sfn::Client,
}

#[cfg(test)]
pub use MockSfnImpl as Sfn;
#[cfg(not(test))]
pub use SfnImpl as Sfn;

#[cfg_attr(test, automock)]
impl SfnImpl {
    #[allow(dead_code)]
    pub fn new(inner: sfn::Client) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    pub async fn describe_state_machine(
        &self,
        sfn_arn: &str,
    ) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
        self.inner
            .describe_state_machine()
            .state_machine_arn(sfn_arn)
            .send()
            .await
    }

    #[allow(dead_code)]
    pub async fn list_tags_for_resource(
        &self,
        sfn_arn: &str,
    ) -> Result<ListTagsForResourceOutput, sfn::error::SdkError<ListTagsForResourceError>> {
        self.inner
            .list_tags_for_resource()
            .resource_arn(sfn_arn)
            .send()
            .await
    }

    pub async fn create_state_machine(
        &self,
        sfn: &StateMachine,
    ) -> Result<CreateStateMachineOutput, sfn::error::SdkError<CreateStateMachineError>> {
        let mut builder = self
            .inner
            .create_state_machine()
            .name(&sfn.name)
            .definition(&sfn.definition)
            .role_arn(&sfn.role_arn)
            .r#type(sfn.r#type.into());

        if let Some(logging_configuration) = &sfn.logging_configuration {
            builder = builder.logging_configuration(logging_configuration.clone().into());
        }
        if let Some(tracing_configuration) = &sfn.tracing_configuration {
            builder = builder.tracing_configuration(tracing_configuration.clone().into());
        }
        for tag in &sfn.tags {
            builder = builder.tags(tag.clone().into());
        }

        // XXX: Handle publish and version_description some day?

        builder.send().await
    }

    pub async fn update_state_machine(
        &self,
        state_arn: &str,
        state: &StateMachine,
    ) -> Result<UpdateStateMachineOutput, sfn::error::SdkError<UpdateStateMachineError>> {
        let mut builder = self
            .inner
            .update_state_machine()
            .state_machine_arn(state_arn)
            .definition(&state.definition)
            .role_arn(&state.role_arn);

        if let Some(logging_configuration) = &state.logging_configuration {
            builder = builder.logging_configuration(logging_configuration.clone().into());
        }
        if let Some(tracing_configuration) = &state.tracing_configuration {
            builder = builder.tracing_configuration(tracing_configuration.clone().into());
        }

        // XXX: Handle publish and version_description some day?

        builder.send().await
    }
}

pub async fn create_state_machine(
    _client: &Sfn,
) -> Result<CreateStateMachineOutput, sfn::error::SdkError<CreateStateMachineError>> {
    todo!()
}

pub async fn update_state_machine(
    _client: &Sfn,
) -> Result<UpdateStateMachineOutput, sfn::error::SdkError<UpdateStateMachineError>> {
    todo!()
}

pub async fn delete_state_machine(
    _client: &Sfn,
) -> Result<DeleteStateMachineOutput, sfn::error::SdkError<DeleteStateMachineError>> {
    todo!()
}

async fn list_tags_for_resource(client: &Sfn, sfn_arn: &str) -> Vec<ResourceTag> {
    let res = client.list_tags_for_resource(sfn_arn).await;

    match res {
        Ok(output) => {
            let tags = output.tags();
            tags.iter()
                .map(|tag| ResourceTag::from(tag.clone()))
                .collect()
        }
        Err(err) => {
            panic!(
                "failed to list tags for resource({}) with error: {}",
                sfn_arn, err
            );
        }
    }
}

pub async fn tag_resource(
    _client: &Sfn,
) -> Result<TagResourceOutput, sfn::error::SdkError<TagResourceError>> {
    todo!()
}

pub async fn untag_resource(
    _client: &Sfn,
) -> Result<UntagResourceOutput, sfn::error::SdkError<UntagResourceError>> {
    todo!()
}

pub async fn describe_state_machine_with_tags(client: &Sfn, sfn_arn: &str) -> Option<StateMachine> {
    let res = client.describe_state_machine(sfn_arn).await;

    match res {
        Ok(output) => {
            let tags = list_tags_for_resource(client, sfn_arn).await;
            let mut sfn = StateMachine::from(output);
            sfn.tags = tags;

            Some(sfn)
        }
        Err(err) => {
            let service_error = err.into_service_error();
            if service_error.is_state_machine_does_not_exist() {
                None
            } else {
                panic!("failed to describe state machine: {}", service_error);
            }
        }
    }
}
