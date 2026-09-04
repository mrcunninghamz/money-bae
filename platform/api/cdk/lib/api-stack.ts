import * as cdk from 'aws-cdk-lib';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import * as apprunner from '@aws-cdk/aws-apprunner-alpha';
import { Construct } from 'constructs';

export class ApiStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Imported by name, not created here - see EcrStack. App Runner requires
    // an image to already exist at 'latest' when the service is created, so
    // the repo has to be its own long-lived stack, deployed (and pushed to)
    // ahead of this one.
    const repo = ecr.Repository.fromRepositoryName(this, 'ApiRepo', 'money-bae-api');
    const dbSecret = secretsmanager.Secret.fromSecretNameV2(this, 'DbSecret', 'money-bae-api/database-url');

    const service = new apprunner.Service(this, 'ApiService', {
      source: apprunner.Source.fromEcr({
        repository: repo,
        tagOrDigest: 'latest',
        imageConfiguration: {
          port: 8080,
          environmentSecrets: { DATABASE_URL: apprunner.Secret.fromSecretsManager(dbSecret) },
        },
      }),
      healthCheck: apprunner.HealthCheck.http({ path: '/health' }),
    });

    new cdk.CfnOutput(this, 'ServiceUrl', { value: service.serviceUrl });
  }
}
