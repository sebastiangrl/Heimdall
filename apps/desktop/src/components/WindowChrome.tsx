import { useCallback, useEffect, useState, type ReactNode } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";

function WindowControl({
  label,
  onClick,
  children,
  danger,
}: {
  label: string;
  onClick: () => void;
  children: ReactNode;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={`flex h-9 w-11 items-center justify-center text-[var(--color-text-secondary)] transition-colors hover:bg-white/8 ${
        danger ? "hover:bg-[var(--color-critical)]/25 hover:text-[var(--color-critical)]" : ""
      }`}
    >
      {children}
    </button>
  );
}

export function WindowChrome() {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri()) return;
    setVisible(true);
    const win = getCurrentWindow();
    const sync = () => {
      void win.isMaximized().then(setMaximized);
    };
    let unlistenResize: (() => void) | undefined;
    let unlistenFocus: (() => void) | undefined;
    sync();
    void (async () => {
      unlistenResize = await win.onResized(sync);
      unlistenFocus = await win.onFocusChanged((focused) => {
        if (focused) sync();
      });
    })();
    return () => {
      unlistenResize?.();
      unlistenFocus?.();
    };
  }, []);

  const onDragDoubleClick = useCallback(async () => {
    const win = getCurrentWindow();
    if (await win.isMaximized()) {
      await win.unmaximize();
    } else {
      await win.maximize();
    }
  }, []);

  if (!visible) return null;

  const win = getCurrentWindow();

  return (
    <header className="window-chrome flex h-9 shrink-0 items-stretch border-b border-white/[0.06] bg-[rgba(9,9,13,0.82)]">
      <div
        data-tauri-drag-region
        onDoubleClick={() => void onDragDoubleClick()}
        className="flex min-w-0 flex-1 items-center gap-2 px-3"
        title={t("window.dragHint")}
      >
        <span className="pointer-events-none font-mono text-[10px] uppercase tracking-[0.2em] text-white/25">
          {t("window.title")}
        </span>
      </div>
      <div className="flex items-stretch">
        <WindowControl label={t("window.minimize")} onClick={() => void win.minimize()}>
          <Minus size={14} strokeWidth={1.75} />
        </WindowControl>
        <WindowControl
          label={maximized ? t("window.restore") : t("window.maximize")}
          onClick={() => void win.toggleMaximize()}
        >
          <Square size={12} strokeWidth={1.75} className={maximized ? "scale-90" : ""} />
        </WindowControl>
        <WindowControl label={t("window.close")} onClick={() => void win.close()} danger>
          <X size={14} strokeWidth={1.75} />
        </WindowControl>
      </div>
    </header>
  );
}