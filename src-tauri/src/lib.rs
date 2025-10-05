use tauri::Manager;
use window_vibrancy::apply_acrylic;

pub mod filesys;
pub mod search;

use crate::search::modals::{
    upload_image_file,
    upload_audio_file,
    upload_document_file
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            upload_image_file,
            upload_audio_file,
            upload_document_file
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Apply acrylic initially
            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((0, 0, 0, 20))).unwrap();

            // Clone window handle for use inside closure
            let win_clone = window.clone();

            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(true) = event {
                    #[cfg(target_os = "windows")]
                    window_vibrancy::apply_acrylic(&win_clone, Some((18, 18, 18, 125))).ok();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
