{
  ssConfigs: [
    {
      state: {
        name: 'fubura-example',
        definition: {
          StartAt: 'RunTask',
          States: {
            Fail: {
              Type: 'Fail',
            },
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
                      Command: [
                        'bundle',
                        'exec',
                        'rails',
                        'routes',
                      ],
                      Name: 'app',
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
          },
        },
        roleArn: 'arn:aws:iam::123456789012:role/fubura_batch',
        type: 'STANDARD',
        loggingConfiguration: {
          level: 'ALL',
          includeExecutionData: true,
          destinations: [
            {
              cloudWatchLogsLogGroup: {
                logGroupArn: 'arn:aws:logs:us-west-2:123456789012:log-group:/sfn/fubura_batch:*',
              },
            },
          ],
        },
        tracingConfiguration: {
          enabled: false,
        },
        tags: [
          {
            key: 'Name',
            value: 'fubura-example',
          },
        ],
      },
      schedule: {
        groupName: 'fubura-group',
        name: 'fubura-example',
        state: 'ENABLED',
        scheduleExpression: 'rate(10 hours)',
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
      },
      deleteAll: false,
      deleteSchedule: false,
    },
  ],
}
