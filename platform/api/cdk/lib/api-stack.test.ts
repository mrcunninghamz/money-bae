import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { ApiStack } from './api-stack';

test('creates an App Runner service with a health check on /health', () => {
  const app = new App();
  const stack = new ApiStack(app, 'TestStack');
  const template = Template.fromStack(stack);

  template.hasResourceProperties('AWS::AppRunner::Service', {
    HealthCheckConfiguration: {
      Path: '/health',
    },
  });
});
