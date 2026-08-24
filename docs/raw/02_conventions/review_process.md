# プルリクエスト (PR) & レビュー運用規約

## 1. 開発フロー
- すべての機能追加、修正、リファクタリングはプルリクエスト (PR) を経由して実施する。
- PR作成・更新をトリガーとしてCI/CDが自動起動し、コード検証およびレビュー資料の自動ビルド・デプロイが行われる。

---

## 2. 自動デプロイされるレビュー資料
PRごとに以下の資産がビルドされ、S3/CloudFront上のPR専用URL (`/review/pull_request/{PR番号}/`) にデプロイされる：
- **HonKitドキュメント**: 設計規約・インフラ・DB構成の閲覧サイト
- **tarpaulinカバレッジレポート**: HTMLカバレッジ結果
- **Rust Doc**: 実装コメントから自動生成されたAPI仕様書
- **Stoplight Elements / RapiDoc**: ブラウザ上で動作確認可能なAPIビューワー
- **Storybook**: フロントエンドコンポーネントカタログ
- **TypeDoc**: TypeScriptドキュメント

---

## 3. GitHub Deployment Status連携
CI/CDは `actions/github-script` を用いて GitHub Deployment Status を更新し、PR画面上に直接レビュー資料のプレビューURLを掲載する。
