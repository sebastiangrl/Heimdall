import type { ReactNode } from "react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Activity,
  Box,
  Brain,
  ChevronLeft,
  ChevronRight,
  Cpu,
  LayoutDashboard,
  Settings,
  Users,
  FolderSearch,
} from "lucide-react";
import { HeimdallLogo } from "@/components/brand/HeimdallLogo";
import { WindowChrome } from "@/components/WindowChrome";
import { useMonitorStatus } from "@/hooks/useMonitorStatus";

const navItems = [
  { key: "overview", icon: LayoutDashboard },
  { key: "hardware", icon: Cpu },
  { key: "processes", icon: Activity },
  { key: "intelligence", icon: Brain },
  { key: "blackbox", icon: Box },
  { key: "investigations", icon: FolderSearch },
  { key: "profiles", icon: Users },
  { key: "settings", icon: Settings },
] as const;

export type NavKey = (typeof navItems)[number]["key"];

const enabledNav: NavKey[] = [
  "overview",
  "hardware",
  "processes",
  "intelligence",
  "blackbox",
  "investigations",
  "profiles",
  "settings",
];

export function Shell({
  children,
  active = "overview",
  onNavigate,
}: {
  children: ReactNode;
  active?: NavKey;
  onNavigate?: (key: NavKey) => void;
}) {
  const { t } = useTranslation();
  const { status: monitor } = useMonitorStatus();
  const [collapsed, setCollapsed] = useState(false);
  const width = collapsed ? "w-[72px]" : "w-60";
  const monitorLabel =
    monitor?.mode === "daemon"
      ? t("monitor.daemonConnected")
      : monitor?.mode === "embedded"
        ? t("monitor.embeddedActive")
        : monitor === null
          ? t("monitor.unknown")
          : t("monitor.unknown");
  const monitorClass =
    monitor?.mode === "daemon"
      ? "text-[var(--color-healthy)]"
      : monitor?.mode === "embedded"
        ? "text-[var(--color-lavender)]"
        : "text-[var(--color-text-secondary)]";

  return (
    <div className="app-backdrop flex h-full min-h-0 flex-col">
      <WindowChrome />
      <div className="flex min-h-0 flex-1">
      <aside
        className={`${width} shrink-0 border-r border-white/5 bg-[color-mix(in_srgb,var(--color-bg-secondary)_90%,transparent)] p-3 transition-[width] duration-300`}
      >
        <div className="flex h-full min-h-0 flex-col gap-4">
          {collapsed ? <HeimdallLogo compact /> : <HeimdallLogo />}

          <nav className="flex flex-1 flex-col gap-1">
            {navItems.map(({ key, icon: Icon }) => {
              const isActive = key === active;
              const enabled = enabledNav.includes(key);
              return (
                <button
                  key={key}
                  type="button"
                  disabled={!enabled}
                  title={t(`nav.${key}`)}
                  onClick={() => enabled && onNavigate?.(key)}
                  className={`flex items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm transition-colors ${
                    isActive
                      ? "border border-[var(--color-violet)]/30 bg-[var(--color-violet)]/15 text-[var(--color-lavender)]"
                      : "text-[var(--color-text-secondary)] hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-40"
                  } ${collapsed ? "justify-center px-2" : ""}`}
                >
                  <Icon size={18} strokeWidth={1.75} />
                  {!collapsed ? <span>{t(`nav.${key}`)}</span> : null}
                </button>
              );
            })}
          </nav>

          <div className="mt-auto space-y-2 border-t border-white/5 pt-3">
            <p className={`text-[10px] uppercase tracking-wide text-[var(--color-text-secondary)] ${collapsed ? "text-center" : ""}`}>
              {collapsed ? "●" : t("sidebar.monitoring")}
            </p>
            {!collapsed ? (
              <p className={`text-xs ${monitorClass}`}>{monitorLabel}</p>
            ) : (
              <p className={`text-center text-lg ${monitorClass}`} title={monitorLabel}>
                ●
              </p>
            )}
            <button
              type="button"
              onClick={() => setCollapsed((c) => !c)}
              className="flex w-full items-center justify-center gap-1 rounded-xl border border-white/5 py-2 text-xs text-[var(--color-text-secondary)] hover:bg-white/5"
              title={collapsed ? t("sidebar.expand") : t("sidebar.collapse")}
            >
              {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
              {!collapsed ? (collapsed ? t("sidebar.expand") : t("sidebar.collapse")) : null}
            </button>
          </div>
        </div>
      </aside>
      <main className="min-h-0 flex-1 overflow-auto p-5 md:p-6">{children}</main>
      </div>
    </div>
  );
}