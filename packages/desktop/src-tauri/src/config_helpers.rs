use std::fs as stdfs;
use std::path::PathBuf;
use tauri::AppHandle;

use std::io::Read;
use tauri::api::path::home_dir;

#[tauri::command]
pub fn backup_and_reset_config() -> Result<String, String> {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cci_config_dir = home.join("cci-config").join("opencode");
    let cci_pref_dir = home.join("cci-tech").join("open-persona-v3");
    let backups_dir = cci_pref_dir.join("backups");

    if let Err(e) = stdfs::create_dir_all(&backups_dir) {
        return Err(format!("Failed to create backups dir: {}", e));
    }

    let cfg = cci_config_dir.join("opencode.jsonc");
    if !cfg.exists() {
        return Err("No config to backup".to_string());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let dstdir = backups_dir.join(timestamp);
    if let Err(e) = stdfs::create_dir_all(&dstdir) {
        return Err(format!("Failed to create dst backup dir: {}", e));
    }

    let dst = dstdir.join("opencode.jsonc");
    if let Err(e) = stdfs::copy(&cfg, &dst) {
        return Err(format!("Failed to copy config to backup: {}", e));
    }

    // Reset to default
    let default = r#"{
  "$schema": "https://opencode.ai/config.json",
  "model": "",
  "small_model": "",
  "provider": {}
}"#;
    if let Err(e) = stdfs::write(&cfg, default) {
        return Err(format!("Failed to write default config: {}", e));
    }

    Ok(format!("Backed up to {:?}", dst))
}
