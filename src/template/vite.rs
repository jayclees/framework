use serde::{Deserialize, Serialize};

/// @see https://github.com/vitejs/vite/blob/main/packages/vite/src/node/plugins/manifest.ts
#[derive(Debug, Serialize, Deserialize)]
pub struct ViteManifestChunk {
    pub src: Option<String>,
    pub file: String,
    pub css: Option<Vec<String>>,
    pub assets: Option<Vec<String>>,
    #[serde(rename = "isEntry")]
    pub is_entry: Option<bool>,
    pub name: Option<String>,
    #[serde(rename = "isDynamicEntry")]
    pub is_dynamic_entry: Option<bool>,
    pub imports: Option<Vec<String>>,
    #[serde(rename = "dynamicImports")]
    pub dynamic_imports: Option<Vec<String>>,
}
