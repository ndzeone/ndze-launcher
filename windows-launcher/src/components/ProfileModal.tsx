import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText, type ExeCandidate, type Game, type GameProfile, type Priority, type ProcessInfo } from "../api";
import { fileName, formatBytes } from "../format";
import { useToast } from "../toast";
import { Toggle } from "./Toggle";

interface Props {
  game: Game;
  onClose: () => void;
  onSaved: (game: Game) => void;
  onRemoved: (id: string) => void;
}

const PRIORITIES: { value: Priority; label: string }[] = [
  { value: "normal", label: "Обычный" },
  { value: "aboveNormal", label: "Выше среднего" },
  { value: "high", label: "Высокий" },
];

export function ProfileModal({ game, onClose, onSaved, onRemoved }: Props) {
  const [profile, setProfile] = useState<GameProfile>(game.profile);
  const [exePath, setExePath] = useState(game.exePath ?? "");
  const [candidates, setCandidates] = useState<ExeCandidate[]>([]);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [newProcess, setNewProcess] = useState("");
  const [saving, setSaving] = useState(false);
  const notify = useToast();

  useEffect(() => {
    api.exeCandidates(game.id).then(setCandidates).catch(() => {});
    api.listProcesses().then(setProcesses).catch(() => {});
  }, [game.id]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const set = <K extends keyof GameProfile>(key: K, value: GameProfile[K]) =>
    setProfile((p) => ({ ...p, [key]: value }));

  const browse = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      defaultPath: game.installDir ?? undefined,
      filters: [{ name: "Исполняемый файл", extensions: ["exe"] }],
    });
    if (path) setExePath(path);
  };

  const addProcess = () => {
    const name = newProcess.trim();
    if (!name) return;
    const normalized = name.toLowerCase().endsWith(".exe") ? name : `${name}.exe`;
    if (!profile.closeProcesses.some((p) => p.toLowerCase() === normalized.toLowerCase())) {
      set("closeProcesses", [...profile.closeProcesses, normalized]);
    }
    setNewProcess("");
  };

  const save = async () => {
    setSaving(true);
    try {
      onSaved(await api.updateGame(game.id, exePath || null, profile));
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    try {
      await api.removeGame(game.id);
      onRemoved(game.id);
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const needsExe = !exePath;
  const exeOptions = candidates.some((c) => c.path === exePath) || !exePath
    ? candidates
    : [{ path: exePath, size: 0 }, ...candidates];

  return (
    <div className="overlay" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal" role="dialog" aria-modal="true" aria-labelledby="profile-title">
        <header className="modal-header">
          <div>
            <h2 id="profile-title">{game.name}</h2>
            <p className="muted small">{game.installDir ?? "Папка не определена"}</p>
          </div>
          <button className="btn icon ghost" onClick={onClose} aria-label="Закрыть">
            ✕
          </button>
        </header>

        <div className="modal-body">
          <section className="field">
            <label className="field-label">Исполняемый файл игры</label>
            <div className="row">
              <select className="grow" value={exePath} onChange={(e) => setExePath(e.target.value)}>
                <option value="">— не выбран —</option>
                {exeOptions.map((c) => (
                  <option key={c.path} value={c.path}>
                    {fileName(c.path)}
                    {c.size ? ` · ${formatBytes(c.size)}` : ""}
                  </option>
                ))}
              </select>
              <button className="btn" onClick={browse}>
                Обзор…
              </button>
            </div>
            <p className="hint">
              {needsExe
                ? "Без exe не работают GPU-настройки, приоритет и схема питания."
                : "Лаунчер отслеживает этот процесс: повышает приоритет после старта и возвращает схему питания после выхода."}
            </p>
          </section>

          <div className="option">
            <div>
              <strong>Высокопроизводительный GPU</strong>
              <p className="hint">Принудительно дискретная видеокарта. Важно для ноутбуков с гибридной графикой.</p>
            </div>
            <Toggle checked={profile.highPerfGpu} onChange={(v) => set("highPerfGpu", v)} disabled={needsExe} label="GPU" />
          </div>

          <div className="option">
            <div>
              <strong>Схема питания на время игры</strong>
              <p className="hint">Переключает на «Высокую/Максимальную производительность» и возвращает прежнюю после выхода.</p>
            </div>
            <Toggle checked={profile.boostPowerPlan} onChange={(v) => set("boostPowerPlan", v)} disabled={needsExe} label="Питание" />
          </div>

          <div className="option">
            <div>
              <strong>Отключить оптимизацию во весь экран (FSO)</strong>
              <p className="hint">Только для старых DX9–11 игр с подтормаживаниями в эксклюзивном fullscreen. В Windows 11 обычно лучше оставить выключенным.</p>
            </div>
            <Toggle checked={profile.disableFso} onChange={(v) => set("disableFso", v)} disabled={needsExe} label="FSO" />
          </div>

          <div className="option">
            <div>
              <strong>Приоритет процесса</strong>
              <p className="hint">«Высокий» помогает при фоновой нагрузке. Античиты (EAC, BattlEye) могут запретить изменение.</p>
            </div>
            <select value={profile.priority} onChange={(e) => set("priority", e.target.value as Priority)} disabled={needsExe}>
              {PRIORITIES.map((p) => (
                <option key={p.value} value={p.value}>
                  {p.label}
                </option>
              ))}
            </select>
          </div>

          <section className="field">
            <label className="field-label">Закрывать перед запуском</label>
            <div className="chips">
              {profile.closeProcesses.map((p) => (
                <button
                  key={p}
                  className="chip removable"
                  onClick={() => set("closeProcesses", profile.closeProcesses.filter((x) => x !== p))}
                  title="Убрать"
                >
                  {p} ✕
                </button>
              ))}
              {profile.closeProcesses.length === 0 && <span className="muted small">Ничего не закрывается</span>}
            </div>
            <div className="row">
              <input
                className="grow"
                list="running-processes"
                placeholder="Например: chrome.exe"
                value={newProcess}
                onChange={(e) => setNewProcess(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addProcess()}
              />
              <datalist id="running-processes">
                {processes.map((p) => (
                  <option key={p.name} value={p.name}>
                    {formatBytes(p.memory)}
                  </option>
                ))}
              </datalist>
              <button className="btn" onClick={addProcess}>
                Добавить
              </button>
            </div>
          </section>

          {game.source.kind === "manual" && (
            <section className="field">
              <label className="field-label">Аргументы запуска</label>
              <input value={profile.launchArgs} placeholder="-dx12 -novid" onChange={(e) => set("launchArgs", e.target.value)} />
            </section>
          )}
        </div>

        <footer className="modal-footer">
          {game.source.kind === "manual" ? (
            <button className="btn danger" onClick={remove}>
              Удалить из библиотеки
            </button>
          ) : (
            <span />
          )}
          <div className="row">
            <button className="btn ghost" onClick={onClose}>
              Отмена
            </button>
            <button className="btn primary" onClick={save} disabled={saving}>
              {saving ? "Сохраняю…" : "Сохранить"}
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
