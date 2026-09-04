import * as cdk from 'aws-cdk-lib';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import { Construct } from 'constructs';

// Kept as its own stack (rather than nested in ApiStack) so the repo
// survives independently of the App Runner service: the service can only
// be created once an image already exists at the tag it references, so the
// repo needs to be deployed - and have an image pushed to it - before
// ApiStack's first deploy.
export class EcrStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    new ecr.Repository(this, 'ApiRepo', { repositoryName: 'money-bae-api' });
  }
}
