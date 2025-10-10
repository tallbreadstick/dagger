use serde::Serialize;

#[derive(Serialize)]
pub struct ResolveResult {
    kind: String,
    value: String,
}

#[tauri::command]
pub fn resolve_path_command(command: &str) -> Result<ResolveResult, String> {
    use std::env;
    use std::path::PathBuf;

    let cmd = command.trim();

    // --- Special virtual path: Home ---
    if cmd.eq_ignore_ascii_case("home") {
        return Ok(ResolveResult {
            kind: "path".into(),
            value: "Home".into(), // internal identifier for Home
        });
    }

    // --- Handle environment variables like %APPDATA%, %LOCALAPPDATA%, etc. ---
    if cmd.starts_with('%') && cmd.ends_with('%') {
        let var_name = &cmd[1..cmd.len() - 1].to_ascii_uppercase();

        let value = match var_name.as_str() {
            "APPDATA" => env::var("APPDATA"),
            "LOCALAPPDATA" => env::var("LOCALAPPDATA"),
            "TEMP" | "TMP" => env::var("TEMP").or_else(|_| env::var("TMP")),
            "USERPROFILE" => env::var("USERPROFILE"),
            _ => return Err(format!("Unknown variable: {}", var_name)),
        }
        .map_err(|_| format!("Environment variable {} not found", var_name))?;

        return Ok(ResolveResult {
            kind: "path".into(),
            value,
        });
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
