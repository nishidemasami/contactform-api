# バックログ・懸念事項・TODO

> 最終更新: 2026-11-27 | ソース: raw/BACKLOG.md, raw/00000011_形式手法による検証.md

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、今後の改善案（TODO）をリストアップする。

---

## バックログ・今後の改善案

### 形式手法検証（Verus / Kani）・`domain` 層関連

1. **【短期】GitHub Actions における Verus / Kani バイナリキャッシュの導入**
   - `validate-ci.yaml` / `api-cicd.yaml` で毎回実行される `cargo install kani-verifier` および Verus リリースアーカイブの展開を `actions/cache` でキャッシュし、CI 実行時間を短縮する。
2. **【中期】新規状態遷移・追加エンドポイントへの Verified Core の適用拡大**
   - メールアドレス構文検証（`trim`後254文字上限・RFC 5321/5322/1123）および制御文字（NUL等）禁止ルールは `validation.rs` にて実装・形式証明済み。今後、問い合わせ状態遷移や新規APIルート・認可ルールを追加する際も、同様に純粋 `no_std` コアで実装し Verus / Kani の証明を追加する。
3. **【長期】`cargo verus` インライン型不変条件（`type_invariant`）および WebAssembly 共有の検討**
   - ワークスペース全体の Rust ツールチェーン更新と `vstd` の `no_std` 対応が成熟した段階で、`value_object.rs` の構造体自身に Verus の型不変条件を持たせる構成への統合を再評価する。
   - `#![no_std]` 化した `domain` クレートを Wasm ビルドし、フロントエンド（`frontend/`）と検証済みバリデーションロジックを共有する構成を検討する。

---

## 関連ページ

- [coding-conventions.md](./coding-conventions.md) — コーディング規約・形式手法ベストプラクティス
- [known-issues.md](./known-issues.md) — 既知の問題
- [overview.md](./overview.md) — プロジェクト概要
