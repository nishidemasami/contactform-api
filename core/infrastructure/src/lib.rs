//! # インフラストラクチャ層
//!
//! ドメイン層のリポジトリtraitを実装する。
//! SeaORMを使用してAurora DSQLやPostgreSQLに接続する。

/// DB接続管理
pub mod db;

/// リポジトリ実装
pub mod repository;
