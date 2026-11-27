import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Disc3, Pause, Play, SkipBack, SkipForward } from "lucide-react";
import { errorText } from "../api";

const releases = "https://github.com/ndzeone/ndze-launcher/releases";
function parts(tag: string) {
  if (!/^v?\d+\.\d+\.\d+$/.test(tag)) throw new Error("Неизвестный формат версии");
  return tag.replace(/^v/, "").split(".").map(Number);
}

export function DesktopTools() {
  const [checking, setChecking] = useState(false);
  const [update, setUpdate] = useState("");
  const [available, setAvailable] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [scan, setScan] = useState("");
  async function check() {
    setChecking(true);
    setAvailable(false);
    setUpdate("");
    try {
      const response = await fetch("https://api.github.com/repos/ndzeone/ndze-launcher/releases/latest", {signal: AbortSignal.timeout(15000), cache: "no-store"});
      if (!response.ok) throw new Error("GitHub вернул HTTP " + response.status);
      const tag = (await response.json()).tag_name;
      const latest = parts(tag);
      const current = parts(await getVersion());
      const difference = latest.map((value, i) => value - (current[i] ?? 0)).find(value => value !== 0);
      const newer = (difference ?? 0) > 0;
      setAvailable(newer);
      setUpdate(newer ? "Доступна версия " + tag : "Установлена актуальная версия.");
    } catch (e) { setUpdate("Не удалось проверить обновления. " + errorText(e)); }
    finally { setChecking(false); }
  }
  async function scanFile() {
    setScanning(true);
    try {
      const path = await open({multiple: false, directory: false});
      if (!path) return;
      setScan("Проверяется " + path);
      setScan(await invoke<string>("defender_scan", {path}));
    } catch (e) { setScan(errorText(e)); }
    finally { setScanning(false); }
  }
  return <div className="card list">
    <div className="list-row"><div className="grow list-text"><strong>Обновления NDZE</strong><span role="status">{update || "Последний стабильный выпуск на GitHub"}</span></div>
      {available && <button className="btn" onClick={() => openUrl(releases + "/latest").catch(e => setUpdate(errorText(e)))}>Скачать</button>}
      <button className="btn" disabled={checking} onClick={check}>{checking ? "Проверяем…" : "Проверить обновления"}</button></div>
    <div className="list-row"><div className="grow list-text"><strong>Microsoft Defender</strong><span role="status">{scan || "Проверить выбранный файл на вирусы"}</span></div>
      <button className="btn" disabled={scanning} onClick={scanFile}>{scanning ? "Проверяем…" : "Выбрать файл"}</button></div>
  </div>;
}

type Track = { title: string; artist: string; playing: boolean };
export function SpotifyPlayer() {
  const [enabled, enable] = useState(() => localStorage.getItem("spotify-player") === "on");
  const [track, setTrack] = useState<Track | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (!enabled || busy) return;
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function refresh() {
      try {
        const next = await invoke<Track | null>("spotify", {action: "status"});
        if (active) { setTrack(next); setError(""); }
      } catch (e) { if (active) { setTrack(null); setError(errorText(e)); } }
      if (active) timer = setTimeout(refresh, 3500);
    }
    void refresh();
    return () => { active = false; clearTimeout(timer); };
  }, [enabled, busy]);
  async function control(action: string) {
    setBusy(true);
    try { setTrack(await invoke<Track | null>("spotify", {action})); }
    catch (e) { setError(errorText(e)); }
    finally { setBusy(false); }
  }
  return <section className="spotify-player">
    <button className="spotify-switch" aria-expanded={enabled} onClick={() => {
      localStorage.setItem("spotify-player", enabled ? "off" : "on");
      enable(!enabled);
    }}><Disc3 size={18}/> Spotify <small>{enabled ? "Свернуть" : "Подключить"}</small></button>
    {enabled && <><strong>{track?.title || "Мини-плеер"}</strong><span>{error || track?.artist || "Включите трек в Spotify на ПК"}</span>
      <div>{([[ "previous", "Предыдущий", SkipBack ], ["toggle", track?.playing ? "Пауза" : "Играть", track?.playing ? Pause : Play], ["next", "Следующий", SkipForward]] as const).map(([action,label,Icon]) =>
        <button key={action} aria-label={label} disabled={!track || busy} onClick={() => control(action)}><Icon size={16}/></button>)}</div></>}
  </section>;
}
