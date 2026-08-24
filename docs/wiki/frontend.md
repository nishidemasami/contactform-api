# フロントエンド規約（Next.js）

> 最終更新: 2026-08-16 | ソース: raw/02_conventions/frontend.md

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

### フォーム要素

- 氏名テキストボックス
- 連絡先テキストボックス（メールアドレス）
- 本文テキストエリア
- 利用規約同意チェックボックス + 利用規約リンク
- 投稿ボタン（問い合わせ投稿APIを実行）

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
