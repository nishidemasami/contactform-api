# バックログ・課題一覧

本ドキュメントは、プロジェクトにおける検討事項、TODO、懸念点、および将来的な改善タスクを管理するためのバックログです。

## 課題・TODO一覧

- [ ] **Liquibase v5 移行検証**
  - JDBCドライバ動的取得コマンド (`liquibase lpm add postgresql`) の検証および CI ワークフローへの反映検討。
- [ ] **CloudFront 定額プラン CloudFormation 対応追随**
  - AWS CloudFormation が CloudFront 定額プランの設定をネイティブサポートした際、`frontend/template.yaml` を更新。
- [ ] **API Lambda メモリおよびスレッド設定の最適化検証**
  - CloudWatch メトリクス (Billed Duration, Max Memory, Cold Start 時間) に基づき、割り当てメモリ (128MB / 256MB / 512MB) およびシングルスレッド/マルチスレッド構成のパフォーマンスチューニングを実施。
