//! # ドメイン層
//!
//! ビジネスロジックの中核となるドメインモデルを定義する。
//! 他の業務層（application, infrastructure, presentation）には依存しない。
//!
//! ## `no_std` 対応
//!
//! 本クレートは `#![no_std]`（`alloc` 使用）として実装されており、
//! 標準ライブラリ（`std`）に依存せずにドメインルールと型を表現する。
//! 特に [`inquiry::validation`] モジュールは `alloc` および外部クレートにも依存しない
//! 純粋な `no_std` 検証コアとして設計されている。
//!
//! ## 形式手法による検証（Verus / Kani）
//!
//! | 検証項目 | コマンド例 |
//! |---|---|
//! | `no_std` ビルド検証 | `cargo check -p domain --target x86_64-unknown-none` |
//! | **Verus**（演繹的検証・任意長） | `verus --crate-type=lib --no-cheating core/domain/src/inquiry/validation.rs` |
//! | **Kani**（有界モデル検査・全 `char` / 値オブジェクト / エンティティ） | `cargo kani -p domain` |

#![no_std]

extern crate alloc;

/// 問い合わせドメイン
pub mod inquiry;
