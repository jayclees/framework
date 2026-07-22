use std::env;
use std::path::PathBuf;
use minijinja::{path_loader, Environment};
use minijinja_autoreload::AutoReloader;

pub fn reloader(root: PathBuf) -> AutoReloader {
    // If DISABLE_AUTORELOAD is set, then the path tracking is disabled.
    let disable_autoreload = env::var("DISABLE_AUTORELOAD").as_deref() == Ok("1");

    // If FAST_AUTORELOAD is set, then fast reloading is enabled.
    let fast_autoreload = env::var("FAST_AUTORELOAD").as_deref() == Ok("1");

    // The closure is invoked every time the environment is outdated to recreate it.
    AutoReloader::new(move |notifier| {
        let template_path = root.join("resource/template");
        dbg!(&template_path);
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
