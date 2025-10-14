use crate::filesys::os::windows::set_system_clipboard;

#[tauri::command]
pub fn copy_items_to_clipboard(paths: Vec<String>) -> Result<(), String> {
    println!("{:?}", paths);
    set_system_clipboard(paths)
}