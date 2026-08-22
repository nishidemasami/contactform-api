# AWS IAM & OIDC ポリシー定義 実装例

## 1. 信頼関係ポリシー (`Trust Policy`) JSON

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Federated": "arn:aws:iam::<AWS_ACCOUNT_ID>:oidc-provider/token.actions.githubusercontent.com"
            },
            "Action": "sts:AssumeRoleWithWebIdentity",
            "Condition": {
                "StringEquals": {
                    "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
                    "token.actions.githubusercontent.com:sub": [
                      "repo:<OIDC_SUB_PREFIX>:ref:refs/heads/develop",
                      "repo:<OIDC_SUB_PREFIX>:ref:refs/heads/release",
                      "repo:<OIDC_SUB_PREFIX>:pull_request"
                    ]
                }
            }
        }
    ]
}
```

---

## 2. 許可ポリシー (`Permissions Policy`) JSON

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "CloudFormationDeployTrigger",
            "Effect": "Allow",
            "Action": [
                "cloudformation:CreateChangeSet",
                "cloudformation:ExecuteChangeSet",
                "cloudformation:DescribeStacks",
                "cloudformation:DescribeStackEvents",
                "cloudformation:DescribeChangeSet",
                "cloudformation:GetTemplateSummary"
            ],
            "Resource": "arn:aws:cloudformation:*:<AWS_ACCOUNT_ID>:stack/<SERVICE_NAME>-*"
        },
        {
            "Sid": "PassRoleToCloudFormation",
            "Effect": "Allow",
            "Action": "iam:PassRole",
            "Resource": "<SAM_DEPLOY_ROLE_ARN>",
            "Condition": {
                "StringEquals": {
                    "iam:PassedToService": "cloudformation.amazonaws.com"
                }
            }
        },
        {
            "Sid": "AllowCreateSLRForDSQL",
            "Effect": "Allow",
            "Action": "iam:CreateServiceLinkedRole",
            "Resource": "arn:aws:iam::*:role/aws-service-role/dsql.amazonaws.com/AWSServiceRoleForDSQL*",
            "Condition": {
                "StringEquals": {
                    "iam:AWSServiceName": "dsql.amazonaws.com"
                }
            }
        }
    ]
}
```
