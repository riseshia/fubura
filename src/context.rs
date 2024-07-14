use crate::scheduler::Scheduler;
use crate::sfn::Sfn;
use crate::sts::Sts;

pub struct Context {
    pub scheduler_client: Scheduler,
    pub sfn_client: Sfn,
    pub sts_client: Sts,
    pub aws_region: String,
    pub targets: Option<Vec<String>>,
}

impl Context {
    #[cfg(not(test))]
    pub async fn async_default() -> Self {
        use aws_config::BehaviorVersion;

        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

        let scheduler_client = Scheduler::new(aws_sdk_scheduler::Client::new(&aws_config));
        let sfn_client = Sfn::new(aws_sdk_sfn::Client::new(&aws_config));
        let sts_client = Sts::new(aws_sdk_sts::Client::new(&aws_config));
        let aws_region = aws_config
            .region()
            .unwrap_or_else(|| panic!("AWS region not set"))
            .to_string();

        Self {
            scheduler_client,
            sfn_client,
            sts_client,
            aws_region,
            targets: None,
        }
    }

    #[cfg(test)]
    pub async fn async_default() -> Self {
        use crate::scheduler::MockSchedulerImpl;
        use crate::sfn::MockSfnImpl;
        use crate::sts::MockStsImpl;

        Self {
            scheduler_client: MockSchedulerImpl::default(),
            sfn_client: MockSfnImpl::default(),
            sts_client: MockStsImpl::default(),
            aws_region: "us-west-2".to_string(),
            targets: None,
        }
    }
}
