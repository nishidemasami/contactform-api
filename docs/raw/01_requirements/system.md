# システム全体要件定義

## 概要
コンタクトフォーム (<https://nishidemasami.github.io/contact/index.html>) からPOSTを受け付ける問い合わせAPIの作成。
管理者はDBを直接参照するため、管理画面などは不要。

- **サービス名**: `contactform-api-public`

## 開発の大まかな流れ
1. 人間により、`/docs/raw/*.md` に、規約や要求、要件、設計方針、議事録、などを投入する。
2. AIにより、LLM-Wikiに従い `/docs/*.md` へドキュメントを作成・更新、プルリクエストを作成する。
3. 人間によりAI生成物をレビューし、承認後マージする。
4. AIにより、`/docs/*.md` に従い実装する。
5. CI/CDにより、自動テストを実施しカバレッジレポートを出力する。`/docs/*.md` やドキュメンテーションコメントをドキュメントに出力し、カバレッジレポートと共にレビュー用資料としてデプロイする。
6. 人間により、実装・レビュー用資料・カバレッジレポートをレビューし、承認後マージする。
7. CI/CDにより、テスト環境にデプロイする。
8. 人間により、テスト環境にて手動テストを実施し、承認後本番環境にデプロイする。

## `AWS::SSM::Parameter` 体系方針

- **パラメータ名の命名規則**: `/{環境名}/{サービス名}/{サブシステム名}/{パラメータ名}`
- **環境名**: `develop` または `release`
- **サービス名**: `contactform-api-public`
- **サブシステム名**: `api`, `db`, `frontend`, `review`, `retained` (※ `auth` は今回無いため省略)

| 概要 | サブシステム名 | パラメータ名 | 登録される値（Value） | 概要 |
|---|---|---|---|---|
| API GatewayのエンドポイントURL | `api` | `ApiGatewayDomain` | `!Select [2, !Split ["/", !GetAtt <HttpApiリソース>.ApiEndpoint]]` | CloudFrontのOrigin設定に使用 |
| Aurora DSQLのエンドポイント | `db` | `DSQLEndpoint` | `!GetAtt <AWS::DSQL::Clusterリソース>.Endpoint` | API実装からの接続に使用 |
| DB（DSQL）アクセス用Lambda実行ロールのARN | `db` | `DbConnectLambdaRole` | `!GetAtt <AWS::IAM::Roleリソース>.Arn` | Lambdaのロールに使用 |
| フロントエンド（テスト用）公開用CloudFrontドメイン名 | `frontend` | `DomainName` | `!GetAtt <CloudFrontリソース>.DomainName` | レビュー資料に記載するために使用 |
| フロントエンド（テスト用）公開用CloudFrontディストリビューションID | `frontend` | `DistributionID` | `!Ref <CloudFrontリソース>` | キャッシュの削除に使用 |
| フロントエンド（テスト用）SPA格納S3バケット名 | `retained` | `FrontendBucketName` | `!Ref <フロントエンドSPA格納S3リソース>` | 静的ファイルのアップロードに使用 |
| フロントエンド（テスト用）SPA格納S3のWebsiteURL | `retained` | `FrontendWebsiteURL` | `!GetAtt <フロントエンドSPA格納S3リソース>.WebsiteURL` | CloudFrontのOrigin設定に使用 |
| レビュー資料格納S3バケット名 | `retained` | `ReviewBucketName` | `!Ref <レビュー資料格納S3>` | 静的ファイルのアップロードに使用 |
| レビュー資料格納S3のWebsiteURL | `retained` | `ReviewWebsiteURL` | `!GetAtt <レビュー資料格納S3>.WebsiteURL` | CloudFrontのOrigin設定に使用 |
