use crate::filesys::os::windows::{get_system_clipboard, set_system_clipboard};

#[tauri::command]
pub fn copy_items_to_clipboard(paths: Vec<String>) -> Result<(), String> {
    println!("{:?}", paths);
    set_system_clipboard(paths)
}

#[tauri::command]
pub fn paste_items_from_clipboard(working_dir: &str) -> Result<(), String> {
    get_system_clipboard()?;
    Ok(())
}