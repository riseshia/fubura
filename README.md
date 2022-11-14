# eber-cli

eber-cli is the CLI for EventBridge schedulER to maintain schedule as a code.
Consider it as a specialized terraform with EventBridge.
eber-cli aims not to have cache of remote state, so it tries to fetch all schedules
on every apply and plan, which is only difference with terraform.

## Install

TBW

## How to use

```
usage: eber <command> [<args> ...]

Commands:
  apply
    apply schedules to EventBridge Scheduler

  plan
    plan schedules from EventBridge Scheduler

  init
    generate schedules bootstrap
```

## Configuration

For managing schedule, eber uses jsonnet to describe expected state.

### `eber-config.jsonnet`

It specifies target schedule groups to be managed by eber.

```jsonnet
{
  targetScheduleGroups: [
    "batch-prod"
  ]
}
```

### `<schedule-group>.jsonnet`

It specifies schedules in given group name by filename.
Notice that this schedule group should be specified by `eber-config.jsonnet`

```jsonnet
[
  # GetSchedule response
  { ... },
  { ... },
]
```
