use std::sync::Arc;

use rayon::ThreadPoolBuilder;

pub mod filesys;
pub mod search;
pub mod util;

use crate::{
    filesys::{
        nav::{
            get_tree_from_root, is_directory, list_directory_contents, open_from_path, resolve_user,
        },
        stream::{stream_directory_contents, StreamState},
    },
    search::modals::{upload_audio_file, upload_document_file, upload_image_file},
    util::{
        caches::{fetch_layout_settings, update_layout_settings},
        cmd::resolve_path_command,
        setup::setup_app_environment,
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(StreamState::default()))
        .manage(Arc::new(
            ThreadPoolBuilder::new().num_threads(8).build().unwrap(),
        ))
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
            // util
            resolve_path_command,
            fetch_layout_settings,
            update_layout_settings
        ])
        .setup(|app| setup_app_environment(app).map_err(|e| e.into()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
