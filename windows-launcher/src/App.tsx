import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { Cpu, Gamepad2, Rocket, Settings as SettingsIcon, Sparkles, Trash2, Zap, type LucideIcon } from "lucide-react";
import { api, type SessionEvent } from "./api";
import { ToastProvider, useToast } from "./toast";
import { Library } from "./pages/Library";
import { MyPC } from "./pages/MyPC";
import { Config } from "./pages/Config";
import { Tweaks } from "./pages/Tweaks";
import { Startup } from "./pages/Startup";
import { Cleanup } from "./pages/Cleanup";
import { Settings } from "./pages/Settings";

export type Page = "library" | "pc" | "config" | "tweaks" | "startup" | "cleanup" | "settings";

const NAV: { group: string; items: { id: Page; label: string; icon: LucideIcon }[] }[] = [
  { group: "Игры", items: [{ id: "library", label: "Библиотека", icon: Gamepad2 }] },
  {
    group: "Компьютер",
    items: [
      { id: "pc", label: "Мой ПК", icon: Cpu },
      { id: "config", label: "Конфигурация", icon: Sparkles },
    ],
  },
  {
    group: "Ускорение",
    items: [
      { id: "tweaks", label: "Оптимизация", icon: Zap },
      { id: "startup", label: "Автозагрузка", icon: Rocket },
      { id: "cleanup", label: "Очистка", icon: Trash2 },
    ],
  },
];

function Shell() {
  const [page, setPage] = useState<Page>("library");
  const [running, setRunning] = useState<string | null>(null);
  const [version, setVersion] = useState("");
  const notify = useToast();

  useEffect(() => {
    api.activeSession().then(setRunning).catch(() => {});
    getVersion().then(setVersion).catch(() => {});
    const unlisten = listen<SessionEvent>("session", ({ payload }) => {
      const done = payload.phase === "ended" || payload.phase === "error";
      setRunning(done ? null : payload.gameId);
      if (payload.message) {
        notify(payload.message, payload.phase === "error" ? "error" : done ? "success" : "info");
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [notify]);

  const navButton = (id: Page, label: string, Icon: LucideIcon) => (
    <button key={id} className={`nav-item ${page === id ? "active" : ""}`} onClick={() => setPage(id)}>
      <Icon size={18} strokeWidth={2} />
      {label}
    </button>
  );

  return (
    <div className="shell">
      <div className="glow glow-a" aria-hidden />
      <div className="glow glow-b" aria-hidden />
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">
            <Zap size={18} fill="currentColor" strokeWidth={0} />
          </span>
          <span className="brand-name">NDZE</span>
          <span className="brand-sub">launcher</span>
        </div>
        <nav>
          {NAV.map((group) => (
            <div key={group.group} className="nav-group">
              <span className="nav-label">{group.group}</span>
              {group.items.map((item) => navButton(item.id, item.label, item.icon))}
            </div>
          ))}
        </nav>
        <div className="sidebar-footer">
          {running && (
            <div className="session-pill">
              <span className="pulse" />
              Игра запущена
            </div>
          )}
          {navButton("settings", "Настройки", SettingsIcon)}
          {version && <span className="version">v{version}</span>}
        </div>
      </aside>
      <main className="content">
        <div key={page} className="page-enter">
          {page === "library" && <Library running={running} onLaunched={setRunning} />}
          {page === "pc" && <MyPC onNavigate={setPage} />}
          {page === "config" && <Config onNavigate={setPage} />}
          {page === "tweaks" && <Tweaks />}
          {page === "startup" && <Startup />}
          {page === "cleanup" && <Cleanup />}
          {page === "settings" && <Settings />}
        </div>
      </main>
    </div>
  );
}

export function App() {
  return (
    <ToastProvider>
      <Shell />
    </ToastProvider>
  );
}
