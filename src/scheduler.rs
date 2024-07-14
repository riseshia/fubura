use aws_sdk_scheduler as scheduler;
use aws_sdk_scheduler::operation::create_schedule::{CreateScheduleError, CreateScheduleOutput};
use aws_sdk_scheduler::operation::delete_schedule::{DeleteScheduleError, DeleteScheduleOutput};
use aws_sdk_scheduler::operation::get_schedule::{GetScheduleError, GetScheduleOutput};
use aws_sdk_scheduler::operation::update_schedule::{UpdateScheduleError, UpdateScheduleOutput};

#[allow(unused_imports)]
use mockall::automock;

use crate::types::Schedule;

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

    pub async fn create_schedule(
        &self,
        schedule: &Schedule,
    ) -> Result<CreateScheduleOutput, scheduler::error::SdkError<CreateScheduleError>> {
        self.inner
            .create_schedule()
            .group_name(&schedule.group_name)
            .name(&schedule.name)
            .set_description(schedule.description.clone())
            .state(schedule.state.clone().into())
            .schedule_expression(schedule.schedule_expression.clone())
            .set_schedule_expression_timezone(schedule.schedule_expression_timezone.clone())
            .set_start_date(schedule.start_date)
            .set_end_date(schedule.end_date)
            .set_flexible_time_window(schedule.flexible_time_window.clone().map(|v| v.into()))
            .set_kms_key_arn(schedule.kms_key_arn.clone())
            .target(schedule.target.clone().into())
            .send()
            .await
    }

    pub async fn update_schedule(
        &self,
        schedule: &Schedule,
    ) -> Result<UpdateScheduleOutput, scheduler::error::SdkError<UpdateScheduleError>> {
        self.inner
            .update_schedule()
            .group_name(&schedule.group_name)
            .name(&schedule.name)
            .set_description(schedule.description.clone())
            .state(schedule.state.clone().into())
            .schedule_expression(schedule.schedule_expression.clone())
            .set_schedule_expression_timezone(schedule.schedule_expression_timezone.clone())
            .set_start_date(schedule.start_date)
            .set_end_date(schedule.end_date)
            .set_flexible_time_window(schedule.flexible_time_window.clone().map(|v| v.into()))
            .set_kms_key_arn(schedule.kms_key_arn.clone())
            .target(schedule.target.clone().into())
            .send()
            .await
    }

    pub async fn delete_schedule(
        &self,
        group_name: &str,
        name: &str,
    ) -> Result<DeleteScheduleOutput, scheduler::error::SdkError<DeleteScheduleError>> {
        self.inner
            .delete_schedule()
            .group_name(group_name)
            .name(name)
            .send()
            .await
    }
}

pub async fn create_schedule(
    _client: &Scheduler,
) -> Result<CreateScheduleOutput, scheduler::error::SdkError<CreateScheduleError>> {
    todo!()
}

pub async fn update_schedule(
    _client: &Scheduler,
) -> Result<UpdateScheduleOutput, scheduler::error::SdkError<UpdateScheduleError>> {
    todo!()
}

pub async fn delete_schedule(
    _client: &Scheduler,
) -> Result<DeleteScheduleOutput, scheduler::error::SdkError<DeleteScheduleError>> {
    todo!()
}

pub async fn get_schedule(client: &Scheduler, schedule_name_with_group: &str) -> Option<Schedule> {
    let (group_name, schedule_name) =
        schedule_name_with_group.split_once('/').unwrap_or_else(|| {
            panic!(
                "invalid schedule name with group: {:?}",
                schedule_name_with_group
            );
        });

    let res = client.get_schedule(group_name, schedule_name).await;

    match res {
        Ok(output) => Some(Schedule::from(output)),
        Err(err) => {
            let service_error = err.into_service_error();
            if service_error.is_resource_not_found_exception() {
                eprintln!("schedule does not exist: {}", schedule_name_with_group);
                None
            } else {
                panic!("failed to get schedule: {}", service_error);
            }
        }
    }
}
