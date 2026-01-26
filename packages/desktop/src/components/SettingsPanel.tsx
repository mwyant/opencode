import { createSignal, Show } from "solid-js"
import { invoke } from "@tauri-apps/api/core"

export default function SettingsPanel() {
  const [open, setOpen] = createSignal(false)
  const [config, setConfig] = createSignal<any>(null)

  async function loadConfig() {
    try {
      const cfg = await invoke<string>("read_opencode_config")
      setConfig(JSON.parse(cfg ?? "{}"))
    } catch (e) {
      console.error("Failed to load config", e)
      setConfig({})
    }
  }

  async function saveConfig() {
    try {
      await invoke("write_opencode_config", { content: JSON.stringify(config(), null, 2) })
      setOpen(false)
    } catch (e) {
      console.error("Failed to save config", e)
    }
  }

  return (
    <div>
      <button onClick={() => { setOpen(!open); if (!config()) loadConfig() }} title="Settings">
        ⚙️
      </button>
      <Show when={open()}>
        <div class="settings-panel">
          <h3>Settings</h3>
          <div>
            <label>Main Model</label>
            <input value={config()?.model ?? ""} onInput={(e: any) => setConfig({ ...config(), model: e.target.value })} />
          </div>
          <div>
            <label>Small Model</label>
            <input value={config()?.small_model ?? ""} onInput={(e: any) => setConfig({ ...config(), small_model: e.target.value })} />
          </div>
          <div>
            <label>Tool Model (optional)</label>
            <input value={config()?.tool_model ?? ""} onInput={(e: any) => setConfig({ ...config(), tool_model: e.target.value })} />
          </div>
          <button onClick={saveConfig}>Save</button>
          <button onClick={() => setOpen(false)}>Close</button>
        </div>
      </Show>
    </div>
  )
}
