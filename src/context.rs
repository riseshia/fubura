use crate::scheduler_client::Scheduler;
use crate::sfn_client::Sfn;

pub struct Context {
    pub sfn_client: Sfn,
    pub scheduler_client: Scheduler,
}

impl Context {
    pub fn new(sfn_client: Sfn, scheduler_client: Scheduler) -> Self {
        Self {
            sfn_client,
            scheduler_client,
        }
    }

    #[cfg(not(test))]
    pub async fn async_default() -> Self {
        use aws_config::BehaviorVersion;

        use crate::{scheduler_client, sfn_client};

        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let sfn_client = sfn_client::Sfn::new(aws_sdk_sfn::Client::new(&aws_config));
        let scheduler_client =
            scheduler_client::Scheduler::new(aws_sdk_scheduler::Client::new(&aws_config));

        Self {
            sfn_client,
            scheduler_client,
        }
    }

    #[cfg(test)]
    pub async fn async_default() -> Self {
        use crate::scheduler_client::MockSchedulerImpl;
        use crate::sfn_client::MockSfnImpl;

        Self {
            sfn_client: MockSfnImpl::default(),
            scheduler_client: MockSchedulerImpl::default(),
        }
    }
}
