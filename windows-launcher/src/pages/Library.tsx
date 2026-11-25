import { useCallback, useEffect, useMemo, useState, type CSSProperties } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Clock, Gamepad2, Play, Plus, RefreshCw, Search, Trophy } from "lucide-react";
import { api, errorText, type Game } from "../api";
import { useToast } from "../toast";
import { ProfileModal } from "../components/ProfileModal";
import { GameCard } from "../components/GameCard";
import { PageHeader } from "../components/ui";
import { coverUrl, heroUrl, hue } from "../covers";
import { formatDuration, relativeTime } from "../format";

interface Props {
  running: string | null;
  onLaunched: (id: string) => void;
}

type Filter = "all" | "favorite" | "steam" | "epic" | "manual";
type Sort = "recent" | "name" | "playtime";

const FILTERS: { id: Filter; label: string }[] = [
  { id: "all", label: "Все" },
  { id: "favorite", label: "Избранное" },
  { id: "steam", label: "Steam" },
  { id: "epic", label: "Epic" },
  { id: "manual", label: "Свои" },
];

const SORTERS: Record<Sort, (a: Game, b: Game) => number> = {
  recent: (a, b) => (b.stats.lastPlayed ?? 0) - (a.stats.lastPlayed ?? 0) || a.name.localeCompare(b.name),
  name: (a, b) => a.name.localeCompare(b.name),
  playtime: (a, b) => b.stats.playtimeSecs - a.stats.playtimeSecs || a.name.localeCompare(b.name),
};

function Hero({ game, running, locked, onLaunch }: { game: Game; running: boolean; locked: boolean; onLaunch: () => void }) {
  const [src, setSrc] = useState(heroUrl(game));
  useEffect(() => setSrc(heroUrl(game)), [game]);
  return (
    <section className="hero" style={{ "--h": hue(game.name) } as CSSProperties}>
      {src && <img className="hero-bg" src={src} alt="" onError={() => setSrc(src === coverUrl(game) ? null : coverUrl(game))} />}
      <div className="hero-shade" />
      <div className="hero-content">
        <span className="eyebrow">{game.stats.lastPlayed ? "Продолжить игру" : "Рекомендуем начать"}</span>
        <h2>{game.name}</h2>
        <div className="hero-stats">
          <span><Clock size={14} /> {formatDuration(game.stats.playtimeSecs)}</span>
          <span><Trophy size={14} /> {game.stats.launches} запусков</span>
          <span>{relativeTime(game.stats.lastPlayed)}</span>
        </div>
        <button className="btn primary lg" onClick={onLaunch} disabled={locked}>
          <Play size={18} fill="currentColor" />
          {running ? "Игра запущена" : "Играть"}
        </button>
      </div>
    </section>
  );
}

export function Library({ running, onLaunched }: Props) {
  const [games, setGames] = useState<Game[]>([]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<Filter>("all");
  const [sort, setSort] = useState<Sort>("recent");
  const [scanning, setScanning] = useState(false);
  const [editing, setEditing] = useState<Game | null>(null);
  const notify = useToast();

  const replace = (game: Game) => setGames((list) => list.map((g) => (g.id === game.id ? game : g)));

  const scan = useCallback(async (silent = false) => {
    setScanning(true);
    try {
      const found = await api.scanGames();
      setGames(found);
      if (!silent) notify(`Найдено игр: ${found.length}`, "success");
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setScanning(false);
    }
  }, [notify]);

  useEffect(() => {
    api.listGames().then((list) => (list.length ? setGames(list) : scan(true))).catch((e) => notify(errorText(e), "error"));
    const unlisten = listen("library-changed", () => api.listGames().then(setGames).catch(() => {}));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [scan, notify]);

  const addManual = async () => {
    const path = await open({ multiple: false, directory: false, filters: [{ name: "Игра", extensions: ["exe"] }] });
    if (!path) return;
    try {
      const game = await api.addGame(path);
      setGames((list) => [...list.filter((g) => g.id !== game.id), game]);
      setEditing(game);
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const launch = async (game: Game) => {
    try {
      await api.launchGame(game.id);
      onLaunched(game.id);
      replace({ ...game, stats: { ...game.stats, launches: game.stats.launches + 1, lastPlayed: Date.now() / 1000 } });
      const settings = await api.getSettings();
      if (settings.minimizeOnLaunch) await getCurrentWindow().minimize();
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const favorite = async (game: Game) => {
    try {
      replace(await api.toggleFavorite(game.id));
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return games
      .filter((g) => (filter === "all" ? true : filter === "favorite" ? g.favorite : g.source.kind === filter))
      .filter((g) => !q || g.name.toLowerCase().includes(q))
      .sort(SORTERS[sort]);
  }, [games, query, filter, sort]);

  const featured = useMemo(
    () => [...games].sort(SORTERS.recent).find((g) => g.stats.lastPlayed) ?? games.find((g) => g.favorite) ?? null,
    [games],
  );
  const totalPlaytime = games.reduce((sum, g) => sum + g.stats.playtimeSecs, 0);

  return (
    <section className="page">
      <PageHeader
        icon={Gamepad2}
        title="Библиотека"
        subtitle={`${games.length} игр${totalPlaytime ? ` · наиграно ${formatDuration(totalPlaytime)}` : ""}`}
        actions={
          <>
            <button className="btn" onClick={() => scan()} disabled={scanning}>
              <RefreshCw size={15} className={scanning ? "spin" : ""} />
              {scanning ? "Сканирую…" : "Сканировать"}
            </button>
            <button className="btn primary" onClick={addManual}>
              <Plus size={16} />
              Добавить игру
            </button>
          </>
        }
      />

      {featured && !query && filter === "all" && (
        <Hero game={featured} running={running === featured.id} locked={running !== null} onLaunch={() => launch(featured)} />
      )}

      <div className="toolbar">
        <div className="segmented">
          {FILTERS.map((f) => (
            <button key={f.id} className={filter === f.id ? "active" : ""} onClick={() => setFilter(f.id)}>
              {f.label}
            </button>
          ))}
        </div>
        <div className="toolbar-right">
          <label className="search">
            <Search size={15} />
            <input placeholder="Поиск игры…" value={query} onChange={(e) => setQuery(e.target.value)} />
          </label>
          <select value={sort} onChange={(e) => setSort(e.target.value as Sort)} aria-label="Сортировка">
            <option value="recent">Недавние</option>
            <option value="name">По названию</option>
            <option value="playtime">По времени в игре</option>
          </select>
        </div>
      </div>

      {visible.length === 0 ? (
        <div className="empty">
          <Gamepad2 size={34} />
          {scanning ? "Ищу установленные игры…" : filter === "favorite" ? "Отмечайте игры звёздочкой — они появятся здесь." : "Ничего не найдено. Добавьте exe вручную."}
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
              onFavorite={() => favorite(game)}
            />
          ))}
        </div>
      )}

      {editing && (
        <ProfileModal
          game={editing}
          onClose={() => setEditing(null)}
          onSaved={(g) => {
            replace(g);
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
