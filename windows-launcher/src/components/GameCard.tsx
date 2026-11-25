import { useState, type CSSProperties } from "react";
import { Clock, Play, Settings2, Star } from "lucide-react";
import type { Game } from "../api";
import { coverUrl, hue } from "../covers";
import { formatDuration, relativeTime } from "../format";

interface Props {
  game: Game;
  running: boolean;
  locked: boolean;
  onLaunch: () => void;
  onEdit: () => void;
  onFavorite: () => void;
}

const SOURCE_LABEL = { steam: "Steam", epic: "Epic", manual: "Свой exe" } as const;

export function GameCard({ game, running, locked, onLaunch, onEdit, onFavorite }: Props) {
  const [imageFailed, setImageFailed] = useState(false);
  const src = coverUrl(game);
  const p = game.profile;
  const chips = [
    p.highPerfGpu && "GPU",
    p.boostPowerPlan && "Питание",
    p.priority !== "normal" && (p.priority === "high" ? "Высокий приоритет" : "Приоритет+"),
    p.disableFso && "FSO off",
    p.closeProcesses.length > 0 && `−${p.closeProcesses.length} проц.`,
  ].filter(Boolean) as string[];

  return (
    <article className={`game ${running ? "running" : ""}`}>
      <div className="cover" style={{ "--h": hue(game.name) } as CSSProperties}>
        {src && !imageFailed ? (
          <img src={src} alt="" loading="lazy" onError={() => setImageFailed(true)} />
        ) : (
          <span className="initials">{game.name.slice(0, 2).toUpperCase()}</span>
        )}
        <span className={`badge source-${game.source.kind}`}>{SOURCE_LABEL[game.source.kind]}</span>
        <button
          className={`fav ${game.favorite ? "on" : ""}`}
          onClick={onFavorite}
          aria-label={game.favorite ? "Убрать из избранного" : "В избранное"}
          title={game.favorite ? "Убрать из избранного" : "В избранное"}
        >
          <Star size={15} fill={game.favorite ? "currentColor" : "none"} />
        </button>
        <button className="cover-play" onClick={onLaunch} disabled={locked} aria-label={`Играть в ${game.name}`}>
          <Play size={26} fill="currentColor" />
        </button>
      </div>
      <div className="game-body">
        <h3 title={game.name}>{game.name}</h3>
        <div className="game-meta">
          <Clock size={13} />
          {game.stats.playtimeSecs > 0 ? formatDuration(game.stats.playtimeSecs) : relativeTime(game.stats.lastPlayed)}
        </div>
        <div className="chips">
          {chips.length ? chips.map((c) => <span key={c} className="chip accent">{c}</span>) : <span className="chip ghost">Профиль по умолчанию</span>}
        </div>
        <div className="game-actions">
          <button className="btn primary grow" onClick={onLaunch} disabled={locked}>
            <Play size={15} fill="currentColor" />
            {running ? "Запущена" : "Играть"}
          </button>
          <button className="btn icon" onClick={onEdit} aria-label="Профиль оптимизации" title="Профиль оптимизации">
            <Settings2 size={17} />
          </button>
        </div>
      </div>
    </article>
  );
}
