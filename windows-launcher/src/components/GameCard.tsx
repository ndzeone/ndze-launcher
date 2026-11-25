import { useState, type CSSProperties } from "react";
import type { Game } from "../api";

interface Props {
  game: Game;
  running: boolean;
  locked: boolean;
  onLaunch: () => void;
  onEdit: () => void;
}

const SOURCE_LABEL = { steam: "Steam", epic: "Epic", manual: "Вручную" } as const;

function cover(game: Game) {
  return game.source.kind === "steam"
    ? `https://cdn.cloudflare.steamstatic.com/steam/apps/${game.source.appId}/header.jpg`
    : null;
}

function hue(text: string) {
  let h = 0;
  for (const c of text) h = (h * 31 + c.charCodeAt(0)) % 360;
  return h;
}

export function GameCard({ game, running, locked, onLaunch, onEdit }: Props) {
  const [imageFailed, setImageFailed] = useState(false);
  const src = cover(game);
  const p = game.profile;
  const chips = [
    p.highPerfGpu && "GPU",
    p.disableFso && "FSO off",
    p.priority !== "normal" && (p.priority === "high" ? "High" : "Above"),
    p.boostPowerPlan && "Power",
    p.closeProcesses.length > 0 && `−${p.closeProcesses.length} проц.`,
  ].filter(Boolean) as string[];

  return (
    <article className={`card game ${running ? "running" : ""}`}>
      <div className="cover" style={{ "--h": hue(game.name) } as CSSProperties}>
        {src && !imageFailed ? (
          <img src={src} alt="" loading="lazy" onError={() => setImageFailed(true)} />
        ) : (
          <span className="initials">{game.name.slice(0, 2).toUpperCase()}</span>
        )}
        <span className={`badge source-${game.source.kind}`}>{SOURCE_LABEL[game.source.kind]}</span>
      </div>
      <div className="game-body">
        <h3 title={game.name}>{game.name}</h3>
        <div className="chips">
          {chips.length ? chips.map((c) => <span key={c} className="chip">{c}</span>) : <span className="muted small">Профиль не настроен</span>}
        </div>
        <div className="game-actions">
          <button className="btn primary grow" onClick={onLaunch} disabled={locked}>
            {running ? "Запущена" : "Играть"}
          </button>
          <button className="btn icon" onClick={onEdit} aria-label="Профиль" title="Профиль оптимизации">
            <svg viewBox="0 0 24 24" aria-hidden>
              <path d="M19.4 13a7.5 7.5 0 0 0 0-2l2-1.6-2-3.4-2.4 1a7 7 0 0 0-1.7-1L15 3.5h-4l-.4 2.5a7 7 0 0 0-1.7 1l-2.4-1-2 3.4L6.6 11a7.5 7.5 0 0 0 0 2l-2 1.6 2 3.4 2.4-1a7 7 0 0 0 1.7 1l.3 2.5h4l.4-2.5a7 7 0 0 0 1.7-1l2.4 1 2-3.4zM13 15.5a3.5 3.5 0 1 1 0-7 3.5 3.5 0 0 1 0 7z" />
            </svg>
          </button>
        </div>
      </div>
    </article>
  );
}
