use std::sync::Arc;

use rayon::ThreadPoolBuilder;

pub mod filesys;
pub mod search;
pub mod util;

use crate::{
    filesys::{
        nav::{get_tree_from_root, is_directory, list_directory_contents, open_from_path, resolve_user},
        stream::{copy_items_to_clipboard, paste_items_from_clipboard, resolve_copy_conflict, stream_directory_contents, CopyStreamState, FileStreamState},
    },
    search::modals::{upload_audio_file, upload_document_file, upload_image_file},
    util::{
        caches::{fetch_layout_settings, update_layout_settings},
        cmd::{resolve_path_command, resolve_quick_access},
        setup::{open_window, setup_app_environment, window_event_handler},
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    let file_stream_state = Arc::new(FileStreamState::default());
    let copy_stream_state = Arc::new(CopyStreamState::new());
    let rayon_thread_pool = Arc::new(ThreadPoolBuilder::new().num_threads(8).build().unwrap());

    tauri::Builder::default()
        // Auto-start plugin
        .plugin(tauri_plugin_autostart::Builder::new()
            .app_name("Dagger File Explorer")
            .build())
        // Single instance hook: any subsequent launch triggers window creation
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // If window exists, show it
            open_window(app);
        }))
        // Managed state
        .manage(file_stream_state)
        .manage(copy_stream_state)
        .manage(rayon_thread_pool)
        // Invoke handlers
        .invoke_handler(tauri::generate_handler![
            // modals
            upload_image_file,
            upload_audio_file,
            upload_document_file,
            // filesys
            get_tree_from_root,
            resolve_user,
            open_from_path,
            list_directory_contents,
            is_directory,
            // stream
            stream_directory_contents,
            copy_items_to_clipboard,
            paste_items_from_clipboard,
            resolve_copy_conflict,
            // util
            resolve_path_command,
            resolve_quick_access,
            fetch_layout_settings,
            update_layout_settings
        ])
        // Setup hook
        .setup(setup_app_environment)
        .on_window_event(window_event_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
