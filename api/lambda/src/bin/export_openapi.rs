//! # OpenAPI仕様書エクスポートバイナリ
//!
//! `utoipa` 定義から `api/openapi.yaml` を生成する。

use std::{env, fs, path::PathBuf};

use presentation::api::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let mut spec: utoipa::openapi::OpenApi = ApiDoc::openapi();
    // Cargo.toml に license 未指定時、utoipa が空文字列 `license.name = ""` を生成して
    // OpenAPI 3.1 スキーマ違反になるのを防ぐ
    if spec
        .info
        .license
        .as_ref()
        .is_some_and(|l: &utoipa::openapi::License| l.name.is_empty())
    {
        spec.info.license = None;
    }

    let yaml: String = spec.to_yaml()?;
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let output_path: PathBuf = manifest_dir.join("../openapi.yaml");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, yaml)?;
    println!("Exported OpenAPI spec to {:?}", output_path);
    Ok(())
}
