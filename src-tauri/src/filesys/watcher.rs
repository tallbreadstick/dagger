use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter};

pub type SharedWatcher = Arc<Mutex<Option<RecommendedWatcher>>>;

pub fn start_file_watcher(app: &AppHandle, paths: Vec<String>) -> SharedWatcher {
    let watcher: SharedWatcher = Arc::new(Mutex::new(None));
    let watcher_clone = watcher.clone();

    let handle = app.clone();
    std::thread::spawn(move || {
        let mut watcher_inner: RecommendedWatcher = RecommendedWatcher::new(
            move |res: Result<Event>| {
                match res {
                    Ok(event) => {
                        // Emit event to all windows
                        let _ =
                            handle.emit("file-change", serde_json::json!({ "paths": event.paths }));
                    }
                    Err(err) => eprintln!("watch error: {:?}", err),
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        for path in &paths {
            if let Err(e) = watcher_inner.watch(Path::new(path), RecursiveMode::Recursive) {
                eprintln!("Failed to watch {}: {:?}", path, e);
            }
        }

        *watcher_clone.lock().unwrap() = Some(watcher_inner);

        // Keep thread alive
        loop {
            std::thread::park();
        }
    });

    watcher
}
