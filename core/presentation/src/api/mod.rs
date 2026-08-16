//! # APIモジュール
//!
//! HTTPリクエスト・レスポンス型およびOpenAPIドキュメントを定義する。

/// リクエスト型
pub mod request;

/// レスポンス型
pub mod response;

/// OpenAPIパス定義
pub mod inquiry_paths;

/// OpenAPIドキュメント定義
pub mod openapi;
