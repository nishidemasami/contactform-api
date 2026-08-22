# 要件定義

> 最終更新: 2026-08-16 | ソース: raw/01_requirements/system.md, raw/01_requirements/api.md, raw/01_requirements/frontend_and_review.md

コンタクトフォーム APIの要件定義をまとめる。

---

## サービス概要

- **サービス名**: `contactform-api-public`
- **目的**: コンタクトフォーム（<https://nishidemasami.github.io/contact/index.html>）からのPOSTを受け付ける問い合わせAPI

## 機能要件

### API
- コンタクトフォームからPOSTリクエストを受け付ける
- 受け付けた問い合わせをDBに保存する
- エンドポイント: `POST /api/v1/inquiry`
- リクエストボディ（JSON）:
  - `name`: 氏名
  - `email`: 連絡先メールアドレス
  - `message`: 本文

### フロントエンド（テスト用・develop環境のみ）
- 簡易的な静的SPAサイト（Next.js）
- 主目的はテスト。Next.jsのリファレンス実装も兼ねる
- **release環境ではデプロイしない**

### レビュー資料（develop環境のみ）
- レビュー用の静的WEBコンテンツ
- **release環境ではデプロイしない**

### 管理画面
- 不要。管理者はDBを直接参照する

## 非機能要件

### セキュリティ
- Lambda実行ロールはINSERT権限のみ付与（SELECT/UPDATE/DELETE権限は付与しない）
- SQLインジェクション対策: プレースホルダー必須（文字列結合によるSQL組み立て厳禁）
- 権限最小化の原則を徹底

### パフォーマンス
- AWS Lambda（メモリ128MB）でのコールドスタート最小化
- シングルスレッド非同期ランタイム（`tokio` `current_thread`）
- INITフェーズでDB接続を確立し、INVOKEフェーズで再利用

### 可用性・スケーラビリティ
- AWS Lambda + Aurora DSQLによるサーバレス構成
- CloudFront + S3によるCDN配信

---

## 関連ページ

- [overview.md](./overview.md) — プロジェクト全体概要
- [api.md](./api.md) — API規約詳細
- [aws-settings.md](./aws-settings.md) — AWS設定・SSMパラメータ
