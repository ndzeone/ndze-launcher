import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText, type Game } from "../api";
import { useToast } from "../toast";
import { ProfileModal } from "../components/ProfileModal";
import { GameCard } from "../components/GameCard";

interface Props {
  running: string | null;
  onLaunched: (id: string) => void;
}

export function Library({ running, onLaunched }: Props) {
  const [games, setGames] = useState<Game[]>([]);
  const [query, setQuery] = useState("");
  const [scanning, setScanning] = useState(false);
  const [editing, setEditing] = useState<Game | null>(null);
  const notify = useToast();

  const scan = async () => {
    setScanning(true);
    try {
      const found = await api.scanGames();
      setGames(found);
      notify(`Найдено игр: ${found.length}`, "success");
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setScanning(false);
    }
  };

  useEffect(() => {
    api
      .listGames()
      .then((list) => (list.length ? setGames(list) : scan()))
      .catch((e) => notify(errorText(e), "error"));
  }, []);

  const addManual = async () => {
    const path = await open({ multiple: false, directory: false, filters: [{ name: "Игра", extensions: ["exe"] }] });
    if (!path) return;
    try {
      const game = await api.addGame(path);
      setGames((list) => [...list.filter((g) => g.id !== game.id), game].sort((a, b) => a.name.localeCompare(b.name)));
      setEditing(game);
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const launch = async (game: Game) => {
    try {
      await api.launchGame(game.id);
      onLaunched(game.id);
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? games.filter((g) => g.name.toLowerCase().includes(q)) : games;
  }, [games, query]);

  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>Библиотека</h1>
          <p className="muted">Steam и Epic Games находятся автоматически. Для каждой игры — свой профиль оптимизации.</p>
        </div>
        <div className="actions">
          <input className="search" placeholder="Поиск…" value={query} onChange={(e) => setQuery(e.target.value)} />
          <button className="btn" onClick={scan} disabled={scanning}>
            {scanning ? "Сканирую…" : "Сканировать"}
          </button>
          <button className="btn primary" onClick={addManual}>
            + Добавить игру
          </button>
        </div>
      </header>

      {visible.length === 0 ? (
        <div className="empty">
          {scanning ? "Ищу установленные игры…" : "Игры не найдены. Добавьте exe вручную."}
        </div>
      ) : (
        <div className="grid">
          {visible.map((game) => (
            <GameCard
              key={game.id}
              game={game}
              running={running === game.id}
              locked={running !== null}
              onLaunch={() => launch(game)}
              onEdit={() => setEditing(game)}
            />
          ))}
        </div>
      )}

      {editing && (
        <ProfileModal
          game={editing}
          onClose={() => setEditing(null)}
          onSaved={(g) => {
            setGames((list) => list.map((x) => (x.id === g.id ? g : x)));
            setEditing(null);
            notify("Профиль сохранён", "success");
          }}
          onRemoved={(id) => {
            setGames((list) => list.filter((x) => x.id !== id));
            setEditing(null);
          }}
        />
      )}
    </section>
  );
}
