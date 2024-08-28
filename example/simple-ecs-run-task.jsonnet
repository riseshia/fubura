local buildDefinition(command) = {
  StartAt: 'RunTask',
  States: {
    RunTask: {
      End: true,
      Parameters: {
        Cluster: 'fubura',
        EnableExecuteCommand: true,
        LaunchType: 'FARGATE',
        NetworkConfiguration: {
          AwsvpcConfiguration: {
            SecurityGroups: [
              'sg-00000000000000000',
            ],
            Subnets: [
              'subnet-00000000000000000',
              'subnet-11111111111111111',
              'subnet-22222222222222222',
            ],
          },
        },
        Overrides: {
          ContainerOverrides: [
            {
              Name: 'app',
              Command: command,
            },
          ],
          Cpu: null,
          Memory: null,
        },
        PropagateTags: 'TASK_DEFINITION',
        TaskDefinition: 'arn:aws:ecs:us-west-2:123456789012:task-definition/fubura-batch',
      },
      Resource: 'arn:aws:states:::ecs:runTask.sync',
      Retry: [
        {
          BackoffRate: 3,
          ErrorEquals: [
            'ECS.AmazonECSException',
          ],
          IntervalSeconds: 5,
          MaxAttempts: 4,
        },
      ],
      Type: 'Task',
    },
    Fail: {
      Type: 'Fail',
    },
  },
};

local buildState(name, definition) = {
  name: name,
  definition: definition,
  roleArn: 'arn:aws:iam::123456789012:role/fubura_sfn',
  type: 'STANDARD',
  loggingConfiguration: {
    level: 'ALL',
    includeExecutionData: true,
    destinations: [
      {
        cloudWatchLogsLogGroup: {
          logGroupArn: 'arn:aws:logs:us-west-2:123456789012:log-group:fubura_batch',
        },
      },
    ],
  },
  tags: [
    {
      key: 'Name',
      value: 'fubura-batch',
    },
  ],
};

local buildSchedule(name, schedule, scheduleEnabled) = {
  groupName: 'fubura-group',
  name: name,
  state: if scheduleEnabled then 'ENABLED' else 'DISABLED',
  scheduleExpression: schedule,
  scheduleExpressionTimezone: 'Asia/Tokyo',
  flexibleTimeWindow: {
    mode: 'OFF',
  },
  target: {
    arn: 'arn:aws:states:us-west-2:123456789012:stateMachine:fubura-example',
    roleArn: 'arn:aws:iam::123456789012:role/fubura_batch',
    deadLetterConfig: {
      arn: 'arn:aws:sqs:us-west-2:123456789012:fubura_batch_dlq',
    },
    input: '{}',
    retryPolicy: {
      maximumEventAgeInSeconds: 86400,
      maximumRetryAttempts: 0,
    },
  },
};

local batch(name, definition, schedule, scheduleEnabled=true) = {
  state: buildState(name, definition),
  schedule: buildSchedule(name, schedule, scheduleEnabled),
};

[
  batch('some-task', buildDefinition(
    command=['bundle', 'exec', 'rails', 'routes']
  ), 'rate(1 hours)'),
]
