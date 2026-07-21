use serde::{Deserialize, Serialize};

/// @see https://github.com/vitejs/vite/blob/main/packages/vite/src/node/plugins/manifest.ts
#[derive(Debug, Serialize, Deserialize)]
pub struct ViteManifestChunk {
    src: Option<String>,
    file: String,
    css: Option<Vec<String>>,
    assets: Option<Vec<String>>,
    #[serde(rename = "isEntry")]
    is_entry: Option<bool>,
    name: Option<String>,
    #[serde(rename = "isDynamicEntry")]
    is_dynamic_entry: Option<bool>,
    imports: Option<Vec<String>>,
    #[serde(rename = "dynamicImports")]
    dynamic_imports: Option<Vec<String>>,
}
