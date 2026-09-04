import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { ApiStack } from './api-stack';

test('creates an ECR repository named money-bae-api', () => {
  const app = new App();
  const stack = new ApiStack(app, 'TestStack');
  const template = Template.fromStack(stack);

  template.hasResourceProperties('AWS::ECR::Repository', {
    RepositoryName: 'money-bae-api',
  });
});

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
