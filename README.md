# fubura

fubura is the CLI for managing Step Function states with EventBridge Scheduler at once.
Consider it as a specialized terraform with Step function & EventBridge Scheduler.

## Features

- Similar schema to AWS resource API
- Write configuration in Jsonnet, which gives us more flexibility than plain JSON

## Why fubura?

- Need low cost job scheduler implemented with Step Function, EventBridge, and ECS
- Need the better way to handle state machine definition than terraform

## Why not fubura?

- Need few, carefully crafted complex state machine
- Want to handle all resources(including resources call by state machine, scheduler dlq, etc) in one place

## Install

### Homebrew (macOS and Linux)

XXX: to be supported

```sh
brew install rieshia/x/fubura
```

### Binary

XXX: to be supported

## How to use

```
Usage: fubura <COMMAND>

Commands:
  apply   apply config
  plan    plan config
  export  export state machine config
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

### `fubura.jsonnet`

```jsonnet
[{
  schedule: {
    // {Create,Update}Schedule API Request Body
  }, // could be null
  state: {
    // {Create,Update}StateMachine API Request Body
  }, // must not be null
}]
```

## License

This project is licensed under MIT License.

And, this project includes software developed by:
- aws-sdk-config: Licensed under the Apache License, Version 2.0.
- aws-sdk-scheduler: Licensed under the Apache License, Version 2.0.
- aws-sdk-sfn: Licensed under the Apache License, Version 2.0.
- rsjsonnet: Partially licensed under the Apache License, Version 2.0.
- similar: Licensed under the Apache License, Version 2.0.
