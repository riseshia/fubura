use std::process::exit;

use aws_sdk_sfn as sfn;
use aws_sdk_sfn::operation::describe_state_machine::{
    DescribeStateMachineError, DescribeStateMachineOutput,
};
use aws_sdk_sfn::operation::list_tags_for_resource::{
    ListTagsForResourceError, ListTagsForResourceOutput,
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
}

pub async fn create_state_machine(
    _client: &Sfn,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    todo!()
}

pub async fn update_state_machine(
    _client: &Sfn,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    todo!()
}

pub async fn delete_state_machine(
    _client: &Sfn,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    todo!()
}

pub async fn list_tags_for_resource(client: &Sfn, sfn_arn: &str) -> Vec<ResourceTag> {
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
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    todo!()
}

pub async fn untag_resource(
    _client: &Sfn,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    todo!()
}

pub async fn describe_state_machine(client: &Sfn, sfn_arn: &str) -> Option<StateMachine> {
    let res = client.describe_state_machine(sfn_arn).await;

    match res {
        Ok(output) => Some(StateMachine::from(output)),
        Err(err) => {
            let service_error = err.into_service_error();
            if service_error.is_state_machine_does_not_exist() {
                None
            } else {
                eprintln!("failed to describe state machine: {}", service_error);
                exit(1);
            }
        }
    }
}
