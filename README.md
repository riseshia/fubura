# fubura

fubura is the CLI for managing Step Function states with EventBridge Scheduler at once.
Consider it as a specialized terraform with Step function & EventBridge Scheduler.

## Features

- Similar schema to AWS resource API
- Write configuration in Jsonnet, which gives us more flexibility than plain JSON

### Why fubura?

- Need low cost job scheduler implemented with Step Function, EventBridge, and ECS
- Need the better way to handle state machine definition than terraform

### Why not fubura?

- Need few, carefully crafted complex state machine
- Want to handle all resources(including resources call by state machine, scheduler dlq, etc) in one place

## Install

Custom tap is available for Homebrew.

### Homebrew (macOS and Linux)

XXX: to be supported

```sh
brew install rieshia/x/fubura
```

or you can download the code from the [release page](https://github.com/riseshia/fubura/releases) and compile from it.

## How to use

```
Usage: fubura <COMMAND>

Commands:
  apply   apply config
  plan    plan config
  import  import state machine to specified config file
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

```jsonnet
// `fubura.jsonnet`
{
  ssConfigs: [{
    state: {
      // CreateStateMachine API Request Params except "versionDescription"
      // https://docs.aws.amazon.com/step-functions/latest/apireference/API_CreateStateMachine.html
    }, // should not be null
    schedule: {
      // CreateSchedule API Request Body except "ActionAfterCompletion", "ClientToken"
      // https://docs.aws.amazon.com/scheduler/latest/APIReference/API_CreateSchedule.html
    }, // could be null
    deleteAll: true // Optional, default is false. If true, delete the state machine and schedule
    deleteSchedule: true // Optional, default is false. If true, delete the schedule
  }]
}
```

Full example configuration is available in [example](./example) directory.

### `delete*` fields

fubura do not have the state which resource is managed by it,
such as terraform's `tfstate` for simplexity.
Instead, fubura provides `delete*` fields to delete the resource,
so you can delete each resource by setting `delete*` field to `true`, and apply it.

## Required IAM permissions

If you want to allow fubura fine-grained permissions, you can start with following policy.

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowStateModification",
      "Effect": "Allow",
      "Action": [
        "states:CreateStateMachine",
        "states:UpdateStateMachine",
        "states:DeleteStateMachine",
        "states:TagResource",
        "states:UntagResource",
      ],
      "Resource": "*"
    },
    {
      "Sid": "AllowScheduleModification",
      "Effect": "Allow",
      "Action": [
        "scheduler:CreateSchedule",
        "scheduler:UpdateSchedule",
        "scheduler:DeleteSchedule",
      ],
      "Resource": "*"
    }
  ]
}
```

Reference:

- [Actions, resources, and condition keys for AWS Step Functions](https://docs.aws.amazon.com/service-authorization/latest/reference/list_awsstepfunctions.html)
- [Actions, resources, and condition keys for Amazon EventBridge Scheduler](https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazoneventbridgescheduler.html)

## License

This project is licensed under MIT License.

And, this project includes software developed by:
- aws-sdk-config: Licensed under the Apache License, Version 2.0.
- aws-sdk-scheduler: Licensed under the Apache License, Version 2.0.
- aws-sdk-sfn: Licensed under the Apache License, Version 2.0.
- aws-sdk-sts: Licensed under the Apache License, Version 2.0.
- rsjsonnet: Partially licensed under the Apache License, Version 2.0.
- similar: Licensed under the Apache License, Version 2.0.
- similar-asserts: Licensed under the Apache License, Version 2.0.
