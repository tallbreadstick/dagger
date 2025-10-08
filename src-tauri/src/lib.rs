use std::sync::Arc;

use rayon::ThreadPoolBuilder;
#[cfg(target_os = "windows")]
use tauri::Emitter;
use tauri::Manager;
use window_vibrancy::{apply_acrylic, clear_acrylic};

pub mod filesys;
pub mod search;
pub mod util;

use crate::{
    filesys::{
        nav::{
            get_tree_from_root,
            list_directory_contents,
            register_recent_access,
            resolve_user
        },
        stream::{
            stream_directory_contents,
            StreamState
        }
    }, search::modals::{
        upload_audio_file,
        upload_document_file,
        upload_image_file
    },
    util::cmd::resolve_path_command
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(StreamState::default()))
        .manage(Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .unwrap()
        ))
        .invoke_handler(tauri::generate_handler![
            // modals
            upload_image_file,
            upload_audio_file,
            upload_document_file,
            // filesys
            register_recent_access,
            list_directory_contents,
            get_tree_from_root,
            resolve_user,
            // stream
            stream_directory_contents,
            // util
            resolve_path_command
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Apply acrylic initially
            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((0, 0, 0, 20))).unwrap();

            // Clone window handle for use inside closure
            let win_clone = window.clone();

            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = win_clone.emit("window-focus", ());
                            apply_acrylic(&win_clone, Some((0, 0, 0, 20))).ok();
                        }
                    }
                    tauri::WindowEvent::Focused(false) => {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = win_clone.emit("window-blur", ());
                            clear_acrylic(&win_clone).ok();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
