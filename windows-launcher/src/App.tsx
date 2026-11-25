import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, type SessionEvent } from "./api";
import { ToastProvider, useToast } from "./toast";
import { Library } from "./pages/Library";
import { Tweaks } from "./pages/Tweaks";
import { SystemPage } from "./pages/System";

type Page = "library" | "tweaks" | "system";

const NAV: { id: Page; label: string; icon: string }[] = [
  { id: "library", label: "Библиотека", icon: "M4 5h16v3H4zm0 5h16v3H4zm0 5h10v3H4z" },
  { id: "tweaks", label: "Оптимизация", icon: "M13 2 4 14h7l-1 8 9-12h-7z" },
  { id: "system", label: "Система", icon: "M4 4h16v11H4zm5 13h6v3H9z" },
];

function Shell() {
  const [page, setPage] = useState<Page>("library");
  const [running, setRunning] = useState<string | null>(null);
  const notify = useToast();

  useEffect(() => {
    api.activeSession().then(setRunning).catch(() => {});
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

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <svg viewBox="0 0 24 24" aria-hidden>
            <path d="M13 2 4 14h7l-1 8 9-12h-7z" />
          </svg>
          <span>NDZE</span>
        </div>
        <nav>
          {NAV.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${page === item.id ? "active" : ""}`}
              onClick={() => setPage(item.id)}
            >
              <svg viewBox="0 0 24 24" aria-hidden>
                <path d={item.icon} />
              </svg>
              {item.label}
            </button>
          ))}
        </nav>
        {running && (
          <div className="session-pill">
            <span className="pulse" />В игре
          </div>
        )}
      </aside>
      <main className="content">
        {page === "library" && <Library running={running} onLaunched={setRunning} />}
        {page === "tweaks" && <Tweaks />}
        {page === "system" && <SystemPage />}
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
