use tauri::AppHandle;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::Command;

fn user_sidecar_dir() -> PathBuf {
    let home = crate::get_home_dir();
    home.join("cci-tech").join("open-persona-v3").join("sidecars")
}

fn platform_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "opencode-windows-x64.zip"
    } else if cfg!(target_os = "macos") {
        "opencode-macos-universal.zip"
    } else {
        "opencode-linux-x64.tar.gz"
    }
}

#[tauri::command]
fn updater_check_and_install(app: AppHandle) -> Result<String, String> {
    // For MVP: download release asset directly from GitHub Releases for user repo (mwyant/opencode)
    // Note: requires network access and public releases.
    let asset = platform_asset_name();
    let url = format!("https://github.com/mwyant/opencode/releases/latest/download/{}", asset);

    let sidecar_dir = user_sidecar_dir();
    if let Err(e) = fs::create_dir_all(&sidecar_dir) {
        return Err(format!("Failed to create sidecar dir: {}", e));
    }

    let dest = sidecar_dir.join(asset);

    // Use curl available on target env; for robustness use reqwest in future.
    let status = Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap(), &url])
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !status.success() {
        return Err(format!("Download failed with status: {}", status));
    }

    // TODO: verify checksum

    // Unpack and place binary — naive implementation for MVP
    // On Windows we expect a zip and will extract opencode.exe
    if cfg!(target_os = "windows") {
        let _ = fs::create_dir_all(sidecar_dir.join("extract"));
        let status = Command::new("powershell")
            .args(["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", dest.display(), sidecar_dir.join("extract").display())])
            .status()
            .map_err(|e| format!("Failed to run powershell expand: {}", e))?;
        if !status.success() {
            return Err("Failed to extract archive".to_string());
        }
        let exe_src = sidecar_dir.join("extract").join("opencode.exe");
        let exe_dst = sidecar_dir.join("opencode.exe");
        fs::copy(&exe_src, &exe_dst).map_err(|e| format!("Failed to copy exe: {}", e))?;
    } else {
        // Linux/mac: extract tar.gz and copy 'opencode'
        let status = Command::new("tar")
            .args(["-xzf", dest.to_str().unwrap(), "-C", sidecar_dir.to_str().unwrap()])
            .status()
            .map_err(|e| format!("Failed to run tar: {}", e))?;
        if !status.success() {
            return Err("Failed to extract tar.gz".to_string());
        }
    }

    Ok(format!("Downloaded and installed {}", asset))
}
