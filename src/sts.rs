use aws_sdk_sts as sts;
use aws_sdk_sts::operation::get_caller_identity::{
    GetCallerIdentityError, GetCallerIdentityOutput,
};

#[allow(unused_imports)]
use mockall::automock;

use crate::context::FuburaContext;
use crate::types::CallerIdentity;

pub struct StsImpl {
    inner: sts::Client,
}

#[cfg(test)]
pub use MockStsImpl as Sts;
#[cfg(not(test))]
pub use StsImpl as Sts;

#[cfg_attr(test, automock)]
impl StsImpl {
    #[allow(dead_code)]
    pub fn new(inner: sts::Client) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    pub async fn get_caller_identity(
        &self,
    ) -> Result<GetCallerIdentityOutput, sts::error::SdkError<GetCallerIdentityError>> {
        self.inner.get_caller_identity().send().await
    }
}

async fn get_caller_identity(client: &Sts) -> CallerIdentity {
    let res = client.get_caller_identity().await;

    match res {
        Ok(output) => CallerIdentity::from(output),
        Err(err) => {
            panic!("failed to get caller identity: {}", err);
        }
    }
}

pub async fn build_state_arn_prefix(context: &FuburaContext) -> String {
    let caller_identity = get_caller_identity(&context.sts_client).await;
    let account_id = caller_identity.account;
    let aws_region = &context.aws_region;

    format!("arn:aws:states:{}:{}:stateMachine:", aws_region, account_id)
}
