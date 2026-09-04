#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { EcrStack } from '../lib/ecr-stack';
import { ApiStack } from '../lib/api-stack';

const app = new cdk.App();
const env = {
  account: process.env.CDK_DEFAULT_ACCOUNT,
  region: process.env.CDK_DEFAULT_REGION,
};

new EcrStack(app, 'MoneyBaeApiEcrStack', { env });
new ApiStack(app, 'MoneyBaeApiStack', { env });
