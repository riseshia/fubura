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
        state_arn: &str,
    ) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
        self.inner
            .describe_state_machine()
            .state_machine_arn(state_arn)
            .send()
            .await
    }

    #[allow(dead_code)]
    pub async fn list_tags_for_resource(
        &self,
        state_arn: &str,
    ) -> Result<ListTagsForResourceOutput, sfn::error::SdkError<ListTagsForResourceError>> {
        self.inner
            .list_tags_for_resource()
            .resource_arn(state_arn)
            .send()
            .await
    }

    pub async fn create_state_machine(
        &self,
        state: &StateMachine,
    ) -> Result<CreateStateMachineOutput, sfn::error::SdkError<CreateStateMachineError>> {
        // XXX: Handle publish and version_description some day?

        let mut builder = self
            .inner
            .create_state_machine()
            .name(&state.name)
            .definition(serde_json::to_string(&state.definition).unwrap())
            .role_arn(&state.role_arn)
            .r#type(state.r#type.into())
            .set_logging_configuration(state.logging_configuration.clone().map(|lc| lc.into()))
            .set_tracing_configuration(state.tracing_configuration.clone().map(|tc| tc.into()));

        for tag in &state.tags {
            builder = builder.tags(tag.clone().into());
        }

        builder.send().await
    }

    pub async fn update_state_machine(
        &self,
        state_arn: &str,
        state: &StateMachine,
    ) -> Result<UpdateStateMachineOutput, sfn::error::SdkError<UpdateStateMachineError>> {
        // XXX: Handle publish and version_description some day?

        self.inner
            .update_state_machine()
            .state_machine_arn(state_arn)
            .definition(serde_json::to_string(&state.definition).unwrap())
            .role_arn(&state.role_arn)
            .set_logging_configuration(state.logging_configuration.clone().map(|lc| lc.into()))
            .set_tracing_configuration(state.tracing_configuration.clone().map(|tc| tc.into()))
            .send()
            .await
    }

    pub async fn delete_state_machine(
        &self,
        state_arn: &str,
    ) -> Result<DeleteStateMachineOutput, sfn::error::SdkError<DeleteStateMachineError>> {
        self.inner
            .delete_state_machine()
            .state_machine_arn(state_arn)
            .send()
            .await
    }

    pub async fn tag_resource(
        &self,
        state_arn: &str,
        tags: &[ResourceTag],
    ) -> Result<TagResourceOutput, sfn::error::SdkError<TagResourceError>> {
        let mut builder = self.inner.tag_resource().resource_arn(state_arn);

        for tag in tags {
            builder = builder.tags(tag.clone().into());
        }

        builder.send().await
    }

    pub async fn untag_resource(
        &self,
        state_arn: &str,
        tags: &[String],
    ) -> Result<UntagResourceOutput, sfn::error::SdkError<UntagResourceError>> {
        let mut builder = self.inner.untag_resource().resource_arn(state_arn);

        for tag in tags {
            builder = builder.tag_keys(tag);
        }

        builder.send().await
    }
}

pub async fn create_state_machine(client: &Sfn, state: &StateMachine) {
    client
        .create_state_machine(state)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "failed to create state machine({}) with error: {}",
                state.name, e
            );
        });
}

pub async fn update_state_machine(client: &Sfn, state_arn: &str, state: &StateMachine) {
    client
        .update_state_machine(state_arn, state)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "failed to update state machine({}) with error: {}",
                state.name, e
            );
        });
}

pub async fn delete_state_machine(client: &Sfn, state_arn: &str) {
    client
        .delete_state_machine(state_arn)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "failed to delete state machine({}) with error: {}",
                state_arn, e
            );
        });
}

async fn list_tags_for_resource(client: &Sfn, state_arn: &str) -> Vec<ResourceTag> {
    let res = client.list_tags_for_resource(state_arn).await;

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
                state_arn, err
            );
        }
    }
}

pub async fn tag_resource(client: &Sfn, state_arn: &str, tags: &[ResourceTag]) {
    client
        .tag_resource(state_arn, tags)
        .await
        .unwrap_or_else(|e| {
            panic!("failed to tag resource with error: {}", e);
        });
}

pub async fn untag_resource(client: &Sfn, state_arn: &str, tags: &[String]) {
    client
        .untag_resource(state_arn, tags)
        .await
        .unwrap_or_else(|e| {
            panic!("failed to untag resource with error: {}", e);
        });
}

pub async fn describe_state_machine_with_tags(
    client: &Sfn,
    state_arn: &str,
) -> Option<StateMachine> {
    let res = client.describe_state_machine(state_arn).await;

    match res {
        Ok(output) => {
            let tags = list_tags_for_resource(client, state_arn).await;
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
