# fubura

fubura is the CLI for managing Step Function state with EventBridge scheduler at once.
Consider it as a specialized terraform with EventBridge.
fubura aims not to have cache of remote state, so it tries to fetch all resources
on every apply and plan, which is only difference with terraform.

## Install

XXX: to be supported

```sh
brew install rieshia/x/fubura
```


## How to use

```
Usage: fubura <COMMAND>

Commands:
  apply  apply schedules to EventBridge Scheduler
  plan   plan schedules from EventBridge Scheduler
  init   generate schedules bootstrap
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## Configuration

For managing schedule, fubura uses jsonnet to describe expected state.

### `fubura-config.jsonnet`

It specifies target schedule groups to be managed by fubura.

```jsonnet
{
  targetScheduleGroups: [
    "batch-prod"
  ]
}
```

### `<schedule-group>.jsonnet`

It specifies schedules in given group name by filename.
Notice that this schedule group should be specified by `fubura-config.jsonnet`

```jsonnet
[
  # GetSchedule response
  { ... },
  { ... },
]
```
