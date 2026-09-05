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
          environmentVariables: {
            // Entra External ID (CIAM) -- see platform/entra-external-id/.
            // Not secrets: an OIDC issuer URL and an app registration's
            // client ID are both public identifiers.
            OIDC_ISSUER_URL:
              'https://6da7bb61-fb8f-4d95-aa49-7808c0b05d51.ciamlogin.com/6da7bb61-fb8f-4d95-aa49-7808c0b05d51/v2.0',
            OIDC_AUDIENCE: 'b035f6c4-d91c-489b-88bd-4342e60463cd',
          },
        },
      }),
      healthCheck: apprunner.HealthCheck.http({ path: '/health' }),
    });

    new cdk.CfnOutput(this, 'ServiceUrl', { value: service.serviceUrl });
  }
}
