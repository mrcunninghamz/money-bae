import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { EcrStack } from './ecr-stack';

test('creates an ECR repository named money-bae-api', () => {
  const app = new App();
  const stack = new EcrStack(app, 'TestStack');
  const template = Template.fromStack(stack);

  template.hasResourceProperties('AWS::ECR::Repository', {
    RepositoryName: 'money-bae-api',
  });
});
