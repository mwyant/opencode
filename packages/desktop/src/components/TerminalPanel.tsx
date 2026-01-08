import { onCleanup, onMount, JSX } from "solid-js";
import { Terminal } from "xterm";
import "xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api/tauri";

const decoder = new TextDecoder();

function getTerminalGeometry(container: HTMLDivElement): { cols: number; rows: number } {
  const charWidth = 9;
  const charHeight = 18;
  return {
    cols: Math.max(2, Math.floor(container.offsetWidth / charWidth)),
    rows: Math.max(2, Math.floor(container.offsetHeight / charHeight))
  };
}

export default function TerminalPanel(props: { class?: string }): JSX.Element {
  let container: HTMLDivElement | undefined;
  let term: Terminal | undefined;
  let ptyId: number | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let cancelOutput = false;

  const spawnPty = async (path: string, cols: number, rows: number) => {
    try {
      const id = await invoke<number>("plugin:pty|spawn", {
        file: path,
        args: [],
        term_name: null,
        cols,
        rows,
        cwd: null,
        env: {
          OPENCODE_CLIENT: "desktop",
          OPENCODE_EXPERIMENTAL_ICON_DISCOVERY: "true"
        },
        encoding: null,
        handle_flow_control: null,
        flow_control_pause: null,
        flow_control_resume: null
      });
      return id;
    } catch (error) {
      console.error("Failed to spawn PTY session", error);
      return undefined;
    }
  };

  const pollPtyOutput = async () => {
    while (!cancelOutput) {
      if (!ptyId) {
        await new Promise((res) => setTimeout(res, 50));
        continue;
      }
      const chunk = await invoke<number[]>("plugin:pty|read", { pid: ptyId });
      if (chunk && chunk.length) {
        term?.write(decoder.decode(new Uint8Array(chunk)));
      }
      await new Promise((res) => setTimeout(res, 30));
    }
  };

  onMount(() => {
    if (!container) return;
    term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "monospace",
      windowsMode: true,
      theme: {
        background: "#1e1e1e",
        foreground: "#e0e0e0"
      }
    });
    term.open(container);

    const { cols, rows } = getTerminalGeometry(container);
    invoke<string>("get_embedded_cli_path")
      .then((path) => spawnPty(path, cols, rows))
      .then((id) => {
        if (id !== undefined) {
          ptyId = id;
        }
      });

    term.onData((data) => {
      if (ptyId) {
        invoke("plugin:pty|write", { pid: ptyId, data }).catch(() => undefined);
      }
    });

    resizeObserver = new ResizeObserver(() => {
      if (!container || !term) return;
      const { cols: newCols, rows: newRows } = getTerminalGeometry(container);
      term.resize(newCols, newRows);
      if (ptyId) {
        invoke("plugin:pty|resize", { pid: ptyId, cols: newCols, rows: newRows }).catch(() => undefined);
      }
    });
    resizeObserver.observe(container);

    pollPtyOutput();
  });

  onCleanup(() => {
    cancelOutput = true;
    if (resizeObserver) {
      resizeObserver.disconnect();
    }
    if (ptyId) {
      invoke("plugin:pty|kill", { pid: ptyId }).catch(() => undefined);
    }
    term?.dispose();
    term = undefined;
    ptyId = undefined;
  });

  return (
    <div
      ref={container}
      class={props.class ?? "tui-terminal-container"}
      style="width:100%;height:100%;background:#1e1e1e;"
    />
  );
}
