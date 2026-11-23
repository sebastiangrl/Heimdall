import { useEffect } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** Esquinas redondeadas (24px) y fondo transparente fuera del marco en ventana Tauri. */
export function useWindowFrame() {
  useEffect(() => {
    if (!isTauri()) return;

    const root = document.documentElement;
    root.classList.add("window-frame");

    const win = getCurrentWindow();
    const syncMaximized = () => {
      void win.isMaximized().then((max) => {
        root.classList.toggle("window-maximized", max);
      });
    };

    let unlistenResize: (() => void) | undefined;
    syncMaximized();
    void (async () => {
      unlistenResize = await win.onResized(syncMaximized);
    })();

    return () => {
      root.classList.remove("window-frame", "window-maximized");
      unlistenResize?.();
    };
  }, []);
}