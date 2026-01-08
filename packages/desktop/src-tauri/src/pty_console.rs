use tauri::{command, State};
use tauri_plugin_pty::{Pty, PtyOptions, ChildId, PtyHandle};
use std::{sync::Arc, collections::HashMap};
use std::sync::Mutex;

#[derive(Default)]
pub struct Pties {
    pub pty: Arc<Mutex<HashMap<String, PtyHandle>>>,
}

#[command]
pub fn start_opencode_cli(state: State<Pties>, cols: u16, rows: u16) -> Result<String, String> {
    // You may need to adjust the actual opencode binary path depending on how bundled.
    let binary_path = "opencode";
    let opts = PtyOptions {
        cols: Some(cols),
        rows: Some(rows),
        ..Default::default()
    };
    let mut pty = Pty::new(binary_path, &[], opts).map_err(|e| format!("failed to spawn pty: {e}"))?;
    let id = pty.id().to_string();
    state.pty.lock().unwrap().insert(id.clone(), pty.handle());
    Ok(id)
}

#[command]
pub fn write_stdin(state: State<Pties>, id: String, input: String) -> Result<(), String> {
    let pty_map = state.pty.lock().unwrap();
    if let Some(handle) = pty_map.get(&id) {
        handle.write(input.as_bytes()).map_err(|e| format!("failed to write: {e}"))
    } else {
        Err("no such PTY id".to_string())
    }
}

#[command]
pub fn resize_pty(state: State<Pties>, id: String, cols: u16, rows: u16) -> Result<(), String> {
    let pty_map = state.pty.lock().unwrap();
    if let Some(handle) = pty_map.get(&id) {
        handle.resize(cols, rows).map_err(|e| format!("failed to resize: {e}"))
    } else {
        Err("no such PTY id".to_string())
    }
}

#[command]
pub async fn read_pty_output(state: State<Pties>, id: String) -> Result<String, String> {
    use futures::StreamExt;
    let pty_map = state.pty.lock().unwrap();
    if let Some(handle) = pty_map.get(&id) {
        let mut reader = handle.reader();
        let mut output = String::new();
        // Read up to one chunk or until timeout
        if let Some(res) = reader.next().await {
            match res {
                Ok(bytes) => output = String::from_utf8_lossy(&bytes).to_string(),
                Err(e) => return Err(format!("failed to read: {e}")),
            }
        }
        Ok(output)
    } else {
        Err("no such PTY id".to_string())
    }
}

#[command]
pub fn close_pty(state: State<Pties>, id: String) -> Result<(), String> {
    let mut pty_map = state.pty.lock().unwrap();
    if let Some(handle) = pty_map.remove(&id) {
        handle.kill().ok();
    }
    Ok(())
}
