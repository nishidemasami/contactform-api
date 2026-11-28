# 要件定義

> 最終更新: 2026-11-27 | ソース: raw/20260816_要件.md

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
- リクエストボディ（JSON、最大 64 KiB）:

| フィールド | 必須 | 前後空白の扱い | 文字数上限の検証対象 | 上限 | 形式・文字種制約 |
|---|---|---|---|---|---|
| `name` | ✓ | 保持（非トリム） | **`trim`前の文字列** | 100文字 | 空白のみ不可、C0/C1制御文字（`\0`, `\r`, `\n`, `\t` 等）不可 |
| `email` | ✓ | 前後空白を `trim` 正規化 | **`trim`後の文字列** | 254文字 | 空白のみ不可、RFC 5321/5322 `dot-atom` + RFC 1123 ドメイン形式 |
| `message` | ✓ | 保持（非トリム） | **`trim`前の文字列** | 5,000文字 | 空白のみ不可、`\t`・`\n`・`\r` 以外のC0/C1制御文字（`\0` 等）不可 |

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
