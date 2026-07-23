use minijinja::{path_loader, Environment, Error, ErrorKind};
use minijinja_autoreload::AutoReloader;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use vite::ViteManifestChunk;
use tokio::io::AsyncReadExt;

pub mod vite;

pub fn reloader(root: PathBuf) -> AutoReloader {
    // If DISABLE_AUTORELOAD is set, then the path tracking is disabled.
    let disable_autoreload = env::var("DISABLE_AUTORELOAD").as_deref() == Ok("1");

    // If FAST_AUTORELOAD is set, then fast reloading is enabled.
    let fast_autoreload = env::var("FAST_AUTORELOAD").as_deref() == Ok("1");

    // The closure is invoked every time the environment is outdated to recreate it.
    AutoReloader::new(move |notifier| {
        let template_path = root.join("resource/template");
        let mut env = Environment::new();
        env.set_loader(path_loader(&template_path));

        if fast_autoreload {
            notifier.set_fast_reload(true);
        }

        // if watch_path is never called, no fs watcher is created
        if !disable_autoreload {
            notifier.watch_path(&template_path, true);
        }

        Ok(env)
    })
}

async fn vite(path: String) -> Result<String, Error> {
    // todo:
    // Cache vite manifest. Probably in AppState with last loaded at. Then
    // check if the file was modified after, and reload if so. Maybe
    // refactor into ViteManifestChunk method or vite.rs helper fn.

    // if local, return http://vite_url:vite_port/path

    let mut file = OpenOptions::new()
        .read(true)
        .open("public/dist/.vite/manifest.json")
        .await
        .unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).await.unwrap();
    let vite_manifest: HashMap<String, ViteManifestChunk> =
        serde_json::from_str(string.as_ref()).unwrap();
    let manifest_chunk = vite_manifest.get(path.as_str());

    match manifest_chunk {
        Some(chunk) => tokio::fs::read_to_string(format!("public/dist/{}", chunk.file))
            .await
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, "Vite: Failed to load asset.")),
        None => Err(Error::new(ErrorKind::InvalidOperation, "Vite: Failed to load asset.")),
    }
}
