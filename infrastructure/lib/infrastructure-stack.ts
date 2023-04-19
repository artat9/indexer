import { Duration, Stack, StackProps } from 'aws-cdk-lib';
import {
  Architecture,
  DockerImageCode,
  DockerImageFunction,
} from 'aws-cdk-lib/aws-lambda';
import { Construct } from 'constructs';
import { Cors, LambdaIntegration, RestApi } from 'aws-cdk-lib/aws-apigateway';
export class InfrastructureStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);
    const deployFunc = new DockerImageFunction(this, 'deploy_canister', {
      code: DockerImageCode.fromImageAsset('..', {
        exclude: [
          'indexer/node_modules',
          'indexer/.dfx',
          'indexer/dist',
          'indexer/src/declarations',
          'indexer/target',
          'infrastructure/cdk.out',
          'infrastructure/bin',
          'infrastructure/lib',
          'infrastructure/node_modules',
          'infrastructure/test',
          'ic-web3/examples',
          'ic-web3/.github',
        ],
      }),
      architecture: Architecture.X86_64,
    });
    const apiGateway = new RestApi(this, 'apiGatewa', {
      restApiName: 'sample',
      defaultCorsPreflightOptions: {
        allowOrigins: Cors.ALL_ORIGINS,
        allowMethods: Cors.ALL_METHODS,
        allowHeaders: Cors.DEFAULT_HEADERS,
        allowCredentials: true,
        statusCode: 200,
      },
    });
    apiGateway.root.addResource('deploy').addMethod(
      'POST',
      new LambdaIntegration(deployFunc, {
        timeout: Duration.seconds(25),
      })
    );

    // The code that defines your stack goes here
    // example resource
    // const queue = new sqs.Queue(this, 'InfrastructureQueue', {
    //   visibilityTimeout: cdk.Duration.seconds(300)
    // });
  }
}
