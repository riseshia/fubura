pub struct CallerIdentity {
    pub account: String,
}

impl From<aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput> for CallerIdentity {
    fn from(output: aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput) -> Self {
        Self {
            account: output.account.unwrap(),
        }
    }
}
