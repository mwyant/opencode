import { onCleanup, onMount, JSX } from "solid-js";
import { Terminal } from "xterm";
import "xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api/tauri";

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
  let ptyId: string | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let cancelOutput = false;

  const pollPtyOutput = async () => {
    while (!cancelOutput) {
      if (!ptyId) {
        await new Promise((res) => setTimeout(res, 50));
        continue;
      }
      const chunk = await invoke<string>("read_pty_output", { id: ptyId });
      if (chunk) {
        term?.write(chunk);
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
    invoke<string>("start_opencode_cli", { cols, rows })
      .then((id) => {
        ptyId = id;
      })
      .catch((error) => {
        console.error("Failed to start PTY session", error);
      });

    term.onData((data: string) => {
      if (ptyId) {
        invoke("write_stdin", { id: ptyId, input: data }).catch(() => undefined);
      }
    });

    resizeObserver = new ResizeObserver(() => {
      if (!container || !term) return;
      const { cols: newCols, rows: newRows } = getTerminalGeometry(container);
      term.resize(newCols, newRows);
      if (ptyId) {
        invoke("resize_pty", { id: ptyId, cols: newCols, rows: newRows }).catch(() => undefined);
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
      invoke("close_pty", { id: ptyId }).catch(() => undefined);
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
