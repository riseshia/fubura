use anyhow::{Result, bail};
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
        schedule: &Schedule,
    ) -> Result<DeleteScheduleOutput, scheduler::error::SdkError<DeleteScheduleError>> {
        self.inner
            .delete_schedule()
            .group_name(&schedule.group_name)
            .name(&schedule.name)
            .send()
            .await
    }
}

pub async fn create_schedule(client: &Scheduler, schedule: &Schedule) -> Result<()> {
    let res = client.create_schedule(schedule).await;

    if let Err(e) = res {
        bail!(
            "failed to create schedule({}) with error: {}",
            schedule.name,
            e.into_service_error()
        );
    }

    Ok(())
}

pub async fn update_schedule(client: &Scheduler, schedule: &Schedule) -> Result<()> {
    let res = client.update_schedule(schedule).await;

    if let Err(e) = res {
        bail!(
            "failed to update schedule with error: {}",
            e.into_service_error()
        );
    }

    Ok(())
}

pub async fn delete_schedule(client: &Scheduler, schedule: &Schedule) -> Result<()> {
    let res = client.delete_schedule(schedule).await;

    if let Err(e) = res {
        bail!(
            "failed to delete schedule with error: {}",
            e.into_service_error()
        );
    }

    Ok(())
}

pub async fn get_schedule(
    client: &Scheduler,
    schedule_name_with_group: &str,
) -> Result<Option<Schedule>> {
    let split_result = schedule_name_with_group.split_once('/');
    let (group_name, schedule_name) = if let Some((group_name, schedule_name)) = split_result {
        (group_name, schedule_name)
    } else {
        bail!(
            "invalid schedule name with group: {:?}",
            schedule_name_with_group
        );
    };

    let res = client.get_schedule(group_name, schedule_name).await;

    let schedule = match res {
        Ok(output) => Some(Schedule::from(output)),
        Err(err) => {
            let service_error = err.into_service_error();
            if service_error.is_resource_not_found_exception() {
                None
            } else {
                bail!("failed to get schedule: {}", service_error);
            }
        }
    };

    Ok(schedule)
}
