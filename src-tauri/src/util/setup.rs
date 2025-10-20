use std::error::Error;

use tauri::{
    menu::{Menu, MenuItem}, tray::{MouseButton, TrayIconBuilder, TrayIconEvent}, webview::Color, App, AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent
};
use window_vibrancy::{apply_acrylic, clear_acrylic};

use crate::util::caches::{load_home_cache, load_layout_cache, SharedHomeCache, SharedLayoutCache};

pub fn setup_app_environment(app: &mut App) -> Result<(), Box<dyn Error>> {
    #[cfg(desktop)]
    setup_autostart(app);
    setup_system_tray(app).expect("Failed to setup system tray!");
    manage_home_cache(app);
    manage_layout_cache(app);
    let paths_to_watch = vec![dirs_next::home_dir().unwrap().to_string_lossy().to_string()];
    let watcher = crate::filesys::watcher::start_file_watcher(&app.handle(), paths_to_watch);
    app.manage(watcher);
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn window_event_handler(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            window.hide().unwrap();
        }
        WindowEvent::Focused(true) => {
            let _ = window.emit("window-focus", ());
            apply_acrylic(&window, Some((0, 0, 0, 20))).ok();
            window.set_background_color(Some(Color(0, 0, 0, 0))).ok();
        }
        WindowEvent::Focused(false) => {
            let _ = window.emit("window-blur", ());
            clear_acrylic(&window).ok();
            window
                .set_background_color(Some(Color(120, 120, 120, 255)))
                .ok();
        }
        _ => {}
    }
}

fn manage_home_cache(app: &mut App) {
    let handle = app.handle();
    let cache = load_home_cache(&handle);
    app.manage(SharedHomeCache::new(cache));
}

fn manage_layout_cache(app: &mut App) {
    let handle = app.handle();
    let cache = load_layout_cache(&handle);
    app.manage(SharedLayoutCache::new(cache));
}

fn setup_system_tray(app: &App) -> Result<(), Box<dyn Error>> {
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let close = MenuItem::with_id(app, "close", "Close", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &close])?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Dagger File Explorer")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                open_window(app);
            }
            "close" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            let app = tray.app_handle();
            match event {
                TrayIconEvent::DoubleClick {
                    id: _,
                    position: _,
                    rect: _,
                    button: MouseButton::Left,
                } => {
                    open_window(app);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

/// Spawns the app window if none available
pub fn open_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
        window.reload().unwrap();
    } else {
        let new_window = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::App("index.html".into()),
        )
        .title("Dagger File Explorer")
        .visible(true)
        .decorations(false)
        .transparent(true)
        .resizable(true)
        .maximized(true)
        .center()
        .build()
        .unwrap();
        new_window.show().unwrap();
        new_window.set_focus().unwrap();
    }
}

/// Setup system autostart
#[cfg(desktop)]
fn setup_autostart(app: &App) {
    use tauri_plugin_autostart::MacosLauncher;
    use tauri_plugin_autostart::ManagerExt;
    let _ = app.handle().plugin(tauri_plugin_autostart::init(
        MacosLauncher::LaunchAgent,
        None
    ));
    let autostart_manager = app.autolaunch();
    let _ = autostart_manager.enable();
}