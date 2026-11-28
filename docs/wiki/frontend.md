# フロントエンド規約（Next.js）

> 最終更新: 2026-11-28 | ソース: raw/00000009_フロントエンド規約と実装例.md

テスト用静的SPAサイト（Next.js）の実装規約。**develop環境のみ。release環境ではデプロイしない。**

---

## 1. 概要

| 項目 | 内容 |
|---|---|
| フレームワーク | Next.js（SSG）|
| 言語 | TypeScript |
| デザイン | Tailwind CSS + マテリアルデザイン |
| ビルドツール | npm |
| 目的 | テスト用SPA・Next.jsリファレンス実装 |
| デプロイ環境 | **developのみ**（releaseはデプロイ不可）|

---

## 2. ページ構成

| ページ | パス | 概要 |
|---|---|---|
| トップページ | `/` | 説明文・各ページへのリンク |
| 問い合わせページ | `/inquiry`（等） | 問い合わせフォーム |
| 利用規約ダイアログ | — | 問い合わせページ内のモーダル |

---

## 3. 問い合わせページ仕様

### フォーム要素とクライアント側検証

- `<form onSubmit={...}>` と送信ボタン（`type="submit"`）で構成し、利用規約同意・送信中ガードを維持しつつ、送信時に `setCustomValidity` + `reportValidity()` およびコンテナ層の `canSubmitInquiry` ガードを実行する（利用規約同意チェックのみオンで未入力・空白のみの状態からAPIが呼ばれることを防止）。
- HTML標準の `maxLength` 属性は UTF-16 コード単位で数えるためサロゲートペア（例: `𠮷` や絵文字）やメールアドレスの前後空白を過剰に制限してしまう。そのため `maxLength` 属性は使用せず、`src/lib/inquiryValidation.ts` にてドメイン層と同一の **Unicode コードポイント（スカラー値）単位** で制限・検証する（IME変換中は変換確定時に切り詰め）。
  - 氏名テキストボックス: `required`、`trim`前 `1..=100` コードポイント（空白のみ・制御文字不可）
  - 連絡先テキストボックス: `inputMode="email"`、`required`、前後の Unicode 空白 `trim` 後 `6..=254` コードポイントかつ RFC 5321/5322/1123 準拠メール構文
  - 本文テキストエリア: `required`、`trim`前 `1..=5000` コードポイント（空白のみ・改行タブ以外の制御文字不可）
  - 利用規約同意チェックボックス + 利用規約リンク（`type="button"`）
  - 投稿ボタン（`type="submit"`、検証通過時のみ問い合わせ投稿APIを実行）

### API通信仕様

```
POST /api/v1/inquiry
Content-Type: application/json

{
  "name": "氏名テキストボックスの文字列",
  "email": "連絡先テキストボックスの文字列",
  "message": "本文テキストエリアの文字列"
}
```

- URL: 絶対パス `/api/*`（CloudFrontでルーティングしているためFQDNは同一）

### 利用規約ダイアログ仕様

- 利用規約リンク押下でモーダル表示
- 画面を覆わない大きさ、右上に閉じるボタン（×）
- 背景を薄くグレーアウト
- 閉じるボタン押下・背景押下でダイアログ消去

---

## 4. 共通UIコンポーネント

| コンポーネント | 仕様 |
|---|---|
| ヘッダー | ページ上部固定。左端にハンバーガーメニュー、中央にページタイトル（長い場合は`text-overflow: ellipsis`）|
| フッター | 著作権表示（万国著作権条約準拠）|
| メニュー | ハンバーガーメニュー押下時に各ページへのリンクを表示 |
| トースター | 成功/失敗メッセージ表示。×ボタンで消える。自動消去なし |

---

## 5. デザイン原則

- マテリアルデザイン
- レスポンシブデザイン（どんな端末でも見やすい）
- ダークモード不要

---

## 6. 実装パターン

- **Presentational and Container Componentsパターン** を採用する（Storybookでのテスト・確認を容易にするため）

---

## 7. ビルド・セキュリティ

- パッケージマネージャー: `npm`
- CI/CDに `npm audit`（脆弱性診断）を組み込む
- ビルドjobの権限を最小化（値受け渡しは `jobs.<job_id>.outputs`、成果物は `upload-artifact` / `download-artifact`）

---

## 8. インフラ構成

詳細は [infrastructure.md](./infrastructure.md) を参照。

- CloudFront + S3（S3 Website Endpoint + Referer制御の簡易構成）
- SPAパスルーティング: CloudFront Custom Error Response または CloudFront Function

---

## 関連ページ

- [infrastructure.md](./infrastructure.md) — インフラ規約（CloudFront・S3）
- [cicd.md](./cicd.md) — CI/CD規約
- [api.md](./api.md) — API仕様
