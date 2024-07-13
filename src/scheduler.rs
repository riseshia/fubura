use std::process::exit;

use aws_sdk_scheduler as scheduler;
use aws_sdk_scheduler::operation::get_schedule::{GetScheduleError, GetScheduleOutput};
use aws_sdk_scheduler::operation::get_schedule_group::{
    GetScheduleGroupError, GetScheduleGroupOutput,
};
use aws_sdk_scheduler::operation::list_tags_for_resource::{
    ListTagsForResourceError, ListTagsForResourceOutput,
};

#[allow(unused_imports)]
use mockall::automock;

use crate::types::{ResourceTag, Schedule};

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

    pub async fn get_schedule(
        &self,
        group_name: &str,
        schedule_name: &str,
    ) -> Result<GetScheduleOutput, scheduler::error::SdkError<GetScheduleError>> {
        self.inner
            .get_schedule()
            .group_name(group_name)
            .name(schedule_name)
            .send()
            .await
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

    pub async fn list_tags_for_resource(
        &self,
        schedule_arn: &str,
    ) -> Result<ListTagsForResourceOutput, scheduler::error::SdkError<ListTagsForResourceError>>
    {
        self.inner
            .list_tags_for_resource()
            .resource_arn(schedule_arn)
            .send()
            .await
    }
}

pub async fn create_schedule(
    _client: &Scheduler,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    todo!()
}

pub async fn update_schedule(
    _client: &Scheduler,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    todo!()
}

pub async fn delete_schedule(
    _client: &Scheduler,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    todo!()
}

pub async fn tag_resource(
    _client: &Scheduler,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    todo!()
}

pub async fn untag_resource(
    _client: &Scheduler,
) -> Result<GetScheduleGroupOutput, scheduler::error::SdkError<GetScheduleGroupError>> {
    todo!()
}

async fn list_tags_for_resource(client: &Scheduler, schedule_arn: &str) -> Vec<ResourceTag> {
    let res = client.list_tags_for_resource(schedule_arn).await;

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
                schedule_arn, err
            );
        }
    }
}

pub async fn get_schedule_with_tags(
    client: &Scheduler,
    schedule_name_with_group: &str,
) -> Option<Schedule> {
    let (group_name, schedule_name) =
        schedule_name_with_group.split_once('/').unwrap_or_else(|| {
            eprintln!(
                "invalid schedule name with group: {:?}",
                schedule_name_with_group
            );
            exit(1);
        });

    let res = client.get_schedule(group_name, schedule_name).await;

    match res {
        Ok(output) => {
            let schedule_arn = output.arn().unwrap();
            let tags = list_tags_for_resource(client, schedule_arn).await;
            let mut schedule = Schedule::from(output);
            schedule.tags = tags;

            Some(schedule)
        }
        Err(err) => {
            let service_error = err.into_service_error();
            if service_error.is_resource_not_found_exception() {
                eprintln!("schedule does not exist: {}", schedule_name_with_group);
                None
            } else {
                eprintln!("failed to get schedule: {}", service_error);
                exit(1);
            }
        }
    }
}

pub async fn get_schedule_group(
    client: &Scheduler,
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

        let res = get_schedule_group(&mock, "HelloWorld").await;
        assert!(res.is_ok());
    }
}
