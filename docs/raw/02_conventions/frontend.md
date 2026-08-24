# フロントエンド 開発・セキュリティ規約

## 1. 概要・技術スタック
- フレームワーク: Next.js (TypeScript)
- スタイリング: Tailwind CSS (レスポンシブデザイン、マテリアルデザイン風)
- 配信構成: 静的SPA (SSG / `next build` & `next export`)
- デプロイ環境: `develop` ステージのみ。`release` ステージにはデプロイしない。

---

## 2. セキュリティ・サプライチェーン攻撃対策
- CI/CDパイプラインに `npm audit` を組み込み脆弱性を自動検出。
- パッケージマネージャーには `npm` を使用。`.npmrc` に `install-strategy=nested` を設定しゴースト依存関係を防止。
- ビルドジョブの権限は最小化し、中間成果物は Actions Artifact (`upload-artifact` / `download-artifact`) 経由で受け渡す。

---

## 3. コンポーネント設計・アーキテクチャ
- Presentational and Container Components パターンを意識し、Storybookでの単体コンポーネント確認を容易にする。
- 通信処理は同一ドメインの絶対パス `/api/*` を呼び出す（CloudFrontでバックエンドAPI Gatewayへルーティング）。
