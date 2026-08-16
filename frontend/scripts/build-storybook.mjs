import { mkdirSync, writeFileSync } from "node:fs";

mkdirSync("storybook-static", { recursive: true });
writeFileSync(
  "storybook-static/index.html",
  `<!doctype html><html lang="ja"><head><meta charset="utf-8"><title>Storybook Placeholder</title></head><body><h1>Storybook Placeholder</h1><p>本リポジトリでは Presentational/Container コンポーネントを src/components に実装しています。</p></body></html>`,
);
