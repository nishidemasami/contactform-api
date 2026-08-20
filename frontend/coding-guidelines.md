# コーディング規約(frontend)

- 実装フレームワークは Next.js（TypeScript）を利用する。
- ページは `frontend/src` 配下に配置し、Presentational and Container Components パターンを意識して実装する。
- API 呼び出しは絶対パス `/api/*` を使用し、`Content-Type: application/json` を付与する。
- デザインは Tailwind CSS を利用し、マテリアルデザイン風かつレスポンシブで実装する（ダークモードは実装しない）。
- CI では `sam validate --lint`、`npx tsc --noEmit`、`npm run lint`、`npm test`、`npm audit` を通す。
