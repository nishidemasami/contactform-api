use std::{env, fs, path::PathBuf};

use presentation::api::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let yaml = ApiDoc::openapi().to_yaml()?;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let output_path = manifest_dir.join("../openapi.yaml");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, yaml)?;
    println!("Exported OpenAPI spec to {:?}", output_path);
    Ok(())
}
