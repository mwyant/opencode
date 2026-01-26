import { createSignal, Show, onMount } from "solid-js"
import { invoke } from "@tauri-apps/api/core"

export default function SettingsPanel() {
  const [open, setOpen] = createSignal(false)
  const [config, setConfig] = createSignal<any>(null)
  const [updating, setUpdating] = createSignal(false)
  const [updateStatus, setUpdateStatus] = createSignal('')

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

  async function updateNow() {
    setUpdating(true)
    setUpdateStatus('Checking for updates...')
    try {
      const res = await invoke<string>('updater_check_and_install')
      setUpdateStatus(res)
    } catch (e) {
      setUpdateStatus('Update failed: ' + (e as any).toString())
    }
    setUpdating(false)
  }

  onMount(() => {
    // Listen for the global event to open settings (dispatched by the existing gear button)
    const handler = () => {
      if (!config()) loadConfig()
      setOpen(true)
    }
    window.addEventListener("open-settings-panel", handler as EventListener)
    return () => window.removeEventListener("open-settings-panel", handler as EventListener)
  })

  return (
    <div>
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
          <div style={{ marginTop: '12px' }}>
            <button onClick={saveConfig}>Save</button>
            <button onClick={() => setOpen(false)}>Close</button>
          </div>

          <div style={{ marginTop: '12px', borderTop: '1px solid #eee', paddingTop: '12px' }}>
            <h4>CLI Updater</h4>
            <div>{updateStatus()}</div>
            <button disabled={updating()} onClick={updateNow}>{updating() ? 'Updating...' : 'Update Now'}</button>
          </div>
        </div>
      </Show>
    </div>
  )
}
