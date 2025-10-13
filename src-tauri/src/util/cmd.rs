use serde::Serialize;
use std::{
    collections::HashMap,
    env::{self, VarError},
    path::PathBuf,
};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct ResolveResult {
    kind: String,
    value: String,
}

#[tauri::command]
pub fn resolve_path_command(command: &str) -> Result<ResolveResult, String> {
    let cmd = command.trim();

    // --- Special virtual path: Home ---
    if cmd.eq_ignore_ascii_case("home") {
        return Ok(ResolveResult {
            kind: "path".into(),
            value: "Home".into(),
        });
    }

    // --- Handle environment variables, including paths like %APPDATA%\Dagger ---
    if cmd.starts_with('%') {
        if let Some(end) = cmd[1..].find('%') {
            let var_name = &cmd[1..1 + end].to_ascii_uppercase();
            let remaining_path = cmd[2 + end..].trim_start_matches(['\\', '/']);

            let env_value = match var_name.as_str() {
                "APPDATA" => env::var("APPDATA"),
                "LOCALAPPDATA" => env::var("LOCALAPPDATA"),
                "TEMP" | "TMP" => env::var("TEMP").or_else(|_| env::var("TMP")),
                "USERPROFILE" => env::var("USERPROFILE"),
                _ => Err(VarError::NotPresent),
            }
            .map_err(|e| format!("{e}"))?; // <--- convert to String here

            let full_path = if remaining_path.is_empty() {
                env_value
            } else {
                format!("{}/{}", env_value, remaining_path)
            };

            let normalized = PathBuf::from(full_path);

            return Ok(ResolveResult {
                kind: "path".into(),
                value: normalized.to_string_lossy().to_string(),
            });
        }
    }

    // --- Handle built-in commands ---
    match cmd.to_ascii_lowercase().as_str() {
        "cmd" => {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("cmd")
                    .spawn()
                    .map_err(|e| format!("Failed to open cmd: {}", e))?;
                return Ok(ResolveResult {
                    kind: "action".into(),
                    value: "Opened Command Prompt".into(),
                });
            }
            #[cfg(not(target_os = "windows"))]
            return Err("cmd is only available on Windows".into());
        }
        "powershell" => {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("powershell")
                    .spawn()
                    .map_err(|e| format!("Failed to open PowerShell: {}", e))?;
                return Ok(ResolveResult {
                    kind: "action".into(),
                    value: "Opened PowerShell".into(),
                });
            }
            #[cfg(not(target_os = "windows"))]
            return Err("powershell is only available on Windows".into());
        }
        "explorer" => {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("explorer")
                    .spawn()
                    .map_err(|e| format!("Failed to open Explorer: {}", e))?;
                return Ok(ResolveResult {
                    kind: "action".into(),
                    value: "Opened File Explorer".into(),
                });
            }
            #[cfg(not(target_os = "windows"))]
            return Err("explorer is only available on Windows".into());
        }
        _ => {}
    }

    // --- Handle normal filesystem path ---
    let path = PathBuf::from(cmd);
    if path.exists() {
        Ok(ResolveResult {
            kind: "path".into(),
            value: path.to_string_lossy().to_string(),
        })
    } else {
        Err(format!("Invalid path or command: {}", cmd))
    }
}

#[tauri::command]
pub fn resolve_quick_access(handle: AppHandle) -> Result<HashMap<String, String>, String> {
    let home = handle
        .path()
        .home_dir()
        .map_err(|_| "Failed to resolve home directory")?;

    let mut map = HashMap::new();

    // 🏠 Home (always present)
    map.insert("Home".to_string(), "Home".to_string());

    #[cfg(target_os = "windows")]
    {
        let append = |base: &PathBuf, sub| base.join(sub).to_string_lossy().to_string();
        map.insert("Documents".to_string(), append(&home, "Documents"));
        map.insert("Downloads".to_string(), append(&home, "Downloads"));
        map.insert("Desktop".to_string(), append(&home, "Desktop"));
        map.insert("Pictures".to_string(), append(&home, "Pictures"));
        map.insert("Music".to_string(), append(&home, "Music"));
        map.insert("Videos".to_string(), append(&home, "Videos"));
    }

    #[cfg(target_os = "linux")]
    {
        use dirs_next;

        if let Some(docs) = dirs_next::document_dir() {
            map.insert("Documents".to_string(), docs.to_string_lossy().to_string());
        }
        if let Some(dl) = dirs_next::download_dir() {
            map.insert("Downloads".to_string(), dl.to_string_lossy().to_string());
        }
        if let Some(desktop) = dirs_next::desktop_dir() {
            map.insert("Desktop".to_string(), desktop.to_string_lossy().to_string());
        }
        if let Some(pics) = dirs_next::picture_dir() {
            map.insert("Pictures".to_string(), pics.to_string_lossy().to_string());
        }
        if let Some(music) = dirs_next::audio_dir() {
            map.insert("Music".to_string(), music.to_string_lossy().to_string());
        }
        if let Some(videos) = dirs_next::video_dir() {
            map.insert("Videos".to_string(), videos.to_string_lossy().to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS uses the same standard structure under ~/
        let append = |base: &PathBuf, sub| base.join(sub).to_string_lossy().to_string();
        map.insert("Documents".to_string(), append(&home, "Documents"));
        map.insert("Downloads".to_string(), append(&home, "Downloads"));
        map.insert("Desktop".to_string(), append(&home, "Desktop"));
        map.insert("Pictures".to_string(), append(&home, "Pictures"));
        map.insert("Music".to_string(), append(&home, "Music"));
        map.insert("Videos".to_string(), append(&home, "Movies")); // mac uses “Movies”
    }

    Ok(map)
}
