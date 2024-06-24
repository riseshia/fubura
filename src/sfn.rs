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

pub async fn fetch_state_machine(
    client: Sfn,
    sfn_arn: &str,
) -> Result<DescribeStateMachineOutput, sfn::error::SdkError<DescribeStateMachineError>> {
    client.describe_state_machine(sfn_arn).await
}

#[cfg(test)]
mod test {
    use super::*;

    use std::time::SystemTime;

    use aws_sdk_sfn::{
        operation::describe_state_machine::builders::DescribeStateMachineOutputBuilder,
        primitives::DateTime, types::StateMachineType,
    };
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_describe_state_machine() {
        let mut mock = MockSfnImpl::default();
        mock.expect_describe_state_machine()
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
                    .creation_date(DateTime::from(SystemTime::now()))
                    .build()
                    .unwrap())
            });

        let res = fetch_state_machine(
            mock,
            "arn:aws:states:us-west-2:123456789012:stateMachine:HelloWorld",
        )
        .await;
        assert!(res.is_ok());
    }
}
