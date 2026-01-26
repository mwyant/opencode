use tauri::api::path::home_dir;
use std::path::PathBuf;
use std::fs as stdfs;

#[tauri::command]
pub fn read_opencode_config() -> Result<String, String> {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cfg = home.join("cci-config").join("opencode").join("opencode.jsonc");
    if !cfg.exists() {
        return Ok(String::from("{}"));
    }
    stdfs::read_to_string(&cfg).map_err(|e| format!("Failed to read config: {}", e))
}

#[tauri::command]
pub fn write_opencode_config(content: String) -> Result<String, String> {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join("cci-config").join("opencode");
    if let Err(e) = stdfs::create_dir_all(&dir) {
        return Err(format!("Failed to create config dir: {}", e));
    }
    let cfg = dir.join("opencode.jsonc");
    stdfs::write(&cfg, content.as_bytes()).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(String::from("ok"))
}
