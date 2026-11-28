//! # APIリクエスト型
//!
//! HTTPリクエストのJSONデシリアライズ用構造体を定義する。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 問い合わせ作成リクエスト
///
/// ## フィールドと正規化・検証仕様
///
/// | フィールド | 型 | 生JSONスキーマ制約 | ドメイン正規化・検証ルール |
/// |---|---|---|---|
/// | `name` | `String` | `1..=100`文字 | 正規化なし（`trim`前 `1..=100`文字・空白のみ不可・制御文字不可） |
/// | `email` | `String` | `6`文字以上（前後空白許容） | 前後のUnicode空白を `trim` 正規化後、`6..=254`文字かつRFC 5321/5322/1123準拠メール形式 |
/// | `message` | `String` | `1..=5000`文字 | 正規化なし（`trim`前 `1..=5000`文字・空白のみ不可・改行タブ以外の制御文字不可） |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateInquiryRequest {
    /// 氏名（`trim`前 1〜100文字、空白のみ不可・制御文字不可）
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,

    /// 連絡先メールアドレス（生入力は前後のUnicode空白を許容。サーバー側で前後 `trim` 正規化後に 6〜254文字・RFC準拠メール形式を検証）
    #[schema(min_length = 6)]
    pub email: String,

    /// 問い合わせ本文（`trim`前 1〜5000文字、空白のみ不可・改行タブ以外の制御文字不可）
    #[schema(min_length = 1, max_length = 5000)]
    pub message: String,
}
