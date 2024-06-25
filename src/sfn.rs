use aws_sdk_sfn as sfn;
use aws_sdk_sfn::operation::describe_state_machine::{
    DescribeStateMachineError, DescribeStateMachineOutput,
};

#[allow(unused_imports)]
use mockall::automock;

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

pub async fn describe_state_machine(
    client: &Sfn,
    sfn_arn: &str,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    client.describe_state_machine(sfn_arn).await
}
