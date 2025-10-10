use tauri::{App, Emitter, Manager, Result, WindowEvent};
use window_vibrancy::{apply_acrylic, clear_acrylic};

use crate::util::caches::{load_home_cache, SharedHomeCache};

pub fn setup_app_environment(app: &mut App) -> Result<()> {
    // 🪟 Setup the acrylic window effect
    #[cfg(target_os = "windows")]
    setup_window_acrylic(app)?;
    // 🏠 Initialize home cache
    manage_home_cache(app)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_window_acrylic(app: &mut App) -> Result<()> {
    let window = app.get_webview_window("main").unwrap();

    // Apply acrylic initially
    apply_acrylic(&window, Some((0, 0, 0, 20))).unwrap();

    let win_clone = window.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::Focused(true) => {
            let _ = win_clone.emit("window-focus", ());
            apply_acrylic(&win_clone, Some((0, 0, 0, 20))).ok();
        }
        WindowEvent::Focused(false) => {
            let _ = win_clone.emit("window-blur", ());
            clear_acrylic(&win_clone).ok();
        }
        _ => {}
    });

    Ok(())
}

fn manage_home_cache(app: &mut App) -> Result<()> {
    let handle = app.handle();
    let cache = load_home_cache(&handle);
    app.manage(SharedHomeCache::new(cache));
    Ok(())
}
