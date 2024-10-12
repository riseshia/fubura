use crate::scheduler::Scheduler;
use crate::sfn::Sfn;
use crate::sts::Sts;

pub struct FuburaContext {
    pub scheduler_client: Scheduler,
    pub sfn_client: Sfn,
    pub sts_client: Sts,
    pub aws_region: String,
    pub targets: Option<Vec<String>>,
    pub json_diff_path: Option<String>,
}

impl FuburaContext {
    #[cfg(not(test))]
    pub async fn async_default() -> Self {
        use std::time::Duration;

        use aws_config::{retry::RetryConfig, BehaviorVersion};

        let max_attempts = 100;
        let max_backoff = 3;

        let retry_config = RetryConfig::standard()
            .with_max_attempts(max_attempts)
            .with_max_backoff(Duration::from_secs(max_backoff));
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .retry_config(retry_config)
            .load()
            .await;

        let scheduler_client = Scheduler::new(aws_sdk_scheduler::Client::new(&aws_config));
        let sfn_client = Sfn::new(aws_sdk_sfn::Client::new(&aws_config));
        let sts_client = Sts::new(aws_sdk_sts::Client::new(&aws_config));
        let aws_region = aws_config
            .region()
            .unwrap_or_else(|| {
                eprintln!("AWS region not set");
                std::process::exit(1);
            })
            .to_string();

        Self {
            scheduler_client,
            sfn_client,
            sts_client,
            aws_region,
            targets: None,
            json_diff_path: None,
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
            json_diff_path: None,
        }
    }
}
