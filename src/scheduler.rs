use aws_sdk_scheduler as scheduler;
use aws_sdk_scheduler::operation::get_schedule_group::{
    GetScheduleGroupError, GetScheduleGroupOutput,
};

#[allow(unused_imports)]
use mockall::automock;

pub struct SchedulerImpl {
    inner: scheduler::Client,
}

#[cfg(test)]
pub use MockSchedulerImpl as Scheduler;
#[cfg(not(test))]
pub use SchedulerImpl as Scheduler;

#[cfg_attr(test, automock)]
impl SchedulerImpl {
    #[allow(dead_code)]
    pub fn new(inner: scheduler::Client) -> Self {
        Self { inner }
    }

    pub async fn get_schedule_group(
        &self,
        group_name: &str,
    ) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
        self.inner
            .get_schedule_group()
            .name(group_name)
            .send()
            .await
    }
}

pub async fn get_schedule_group(
    client: Scheduler,
    group_name: &str,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    client.get_schedule_group(group_name).await
}

#[cfg(test)]
mod test {
    use super::*;

    use aws_sdk_scheduler::operation::get_schedule_group::builders::GetScheduleGroupOutputBuilder;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_describe_state_machine() {
        let mut mock = MockSchedulerImpl::default();
        mock.expect_get_schedule_group()
            .with(eq("HelloWorld"))
            .return_once(|_| {
                Ok(GetScheduleGroupOutputBuilder::default()
                    .name("HelloWorld".to_string())
                    .build())
            });

        let res = get_schedule_group(mock, "HelloWorld").await;
        assert!(res.is_ok());
    }
}
