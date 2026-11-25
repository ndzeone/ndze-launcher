import type { Game } from "./api";

const STEAM_CDN = "https://cdn.cloudflare.steamstatic.com/steam/apps";

export const coverUrl = (game: Game) =>
  game.source.kind === "steam" ? `${STEAM_CDN}/${game.source.appId}/header.jpg` : null;

export const heroUrl = (game: Game) =>
  game.source.kind === "steam" ? `${STEAM_CDN}/${game.source.appId}/library_hero.jpg` : null;

export function hue(text: string) {
  let h = 0;
  for (const c of text) h = (h * 31 + c.charCodeAt(0)) % 360;
  return h;
}
