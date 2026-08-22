# AWS SAM テンプレート (`template.yaml`) 実装例

## 1. `api/template.yaml` 実装例

```yaml
Parameters:
  ServiceName:
    Type: String
    Default: contactform-api-public
    Description: "The name of the Service"
  Stage:
    Type: String
    Default: develop
    AllowedValues:
      - develop
      - release
    Description: "Target deployment environment stage (develop or release)"
  SubSystem:
    Type: String
    Default: api
    AllowedValues:
      - api
    Description: "The name of the subsystem being deployed"
  DbConnectLambdaRole:
    Type: AWS::SSM::Parameter::Value<String>
  DSQLEndpoint:
    Type: AWS::SSM::Parameter::Value<String>
  CORSOrigin:
    Type: String

Resources:
  ApiFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: ./lambda
      Handler: bootstrap
      Runtime: provided.al2023
      Architectures:
        - arm64
      Timeout: 10
      MemorySize: 128
      Environment:
        Variables:
          DSQL_ENDPOINT: !Ref DSQLEndpoint
      Role: !Ref DbConnectLambdaRole
      Events:
        ApiEvent:
          Type: HttpApi
          Properties:
            ApiId: !Ref HttpApiGateway
            Path: /{proxy+}
            Method: ANY
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: bootstrap

  HttpApiGateway:
    Type: AWS::Serverless::HttpApi
    Properties:
      DefaultRouteSettings:
        ThrottlingRateLimit: 1
        ThrottlingBurstLimit: 100
      CorsConfiguration:
        AllowOrigins:
          - !Ref CORSOrigin
        AllowMethods:
          - GET
          - POST
          - OPTIONS
        AllowHeaders:
          - Content-Type
        AllowCredentials: false

  ApiGatewayDomainParameter:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub "/${Stage}/${ServiceName}/${SubSystem}/ApiGatewayDomain"
      Type: String
      Value: !Select [2, !Split ["/", !GetAtt HttpApiGateway.ApiEndpoint]]
      Description: "API Gateway Endpoint URL"
```

---

## 2. `db/template.yaml` 実装例

```yaml
Resources:
  DsqlCluster:
    Type: AWS::DSQL::Cluster
    DeletionPolicy: !If [IsRelease, Retain, Delete]
    UpdateReplacePolicy: !If [IsRelease, Retain, Delete]

  DbConnectLambdaRole:
    Type: AWS::IAM::Role
    DeletionPolicy: !If [IsRelease, Retain, Delete]
    UpdateReplacePolicy: !If [IsRelease, Retain, Delete]
    Properties:
      AssumeRolePolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Effect: Allow
            Principal:
              Service:
                - lambda.amazonaws.com
            Action:
              - sts:AssumeRole
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
      Policies:
        - PolicyName: dsql-db-connect
          PolicyDocument:
            Version: '2012-10-17'
            Statement:
              - Effect: Allow
                Action:
                  - dsql:DbConnect
                Resource: !Sub arn:aws:dsql:${AWS::Region}:${AWS::AccountId}:cluster/${DsqlCluster}

  DbConnectLambdaRoleParameter:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub "/${Stage}/${ServiceName}/${SubSystem}/DbConnectLambdaRole"
      Type: String
      Value: !GetAtt DbConnectLambdaRole.Arn
      Description: "DbConnectLambdaRole"

  DSQLEndpointParameter:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub "/${Stage}/${ServiceName}/${SubSystem}/DSQLEndpoint"
      Type: String
      Value: !GetAtt DsqlCluster.Endpoint
      Description: "DSQL Endpoint"
```

---

## 3. `retained/template.yaml` 実装例

```yaml
Resources:
  FrontendBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: !If [IsRelease, Retain, Delete]
    UpdateReplacePolicy: !If [IsRelease, Retain, Delete]
    Properties:
      PublicAccessBlockConfiguration:
        BlockPublicPolicy: false
        RestrictPublicBuckets: false
        BlockPublicAcls: true
        IgnorePublicAcls: true
      WebsiteConfiguration:
        IndexDocument: index.html
        ErrorDocument: index.html

  ReviewBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: !If [IsRelease, Retain, Delete]
    UpdateReplacePolicy: !If [IsRelease, Retain, Delete]
    Properties:
      PublicAccessBlockConfiguration:
        BlockPublicPolicy: false
        RestrictPublicBuckets: false
        BlockPublicAcls: true
        IgnorePublicAcls: true
      WebsiteConfiguration:
        IndexDocument: index.html
        ErrorDocument: error.html

  FrontendBucketNameParameter:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub "/${Stage}/${ServiceName}/${SubSystem}/FrontendBucketName"
      Type: String
      Value: !Ref FrontendBucket

  ReviewBucketNameParameter:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub "/${Stage}/${ServiceName}/${SubSystem}/ReviewBucketName"
      Type: String
      Value: !Ref ReviewBucket
```
