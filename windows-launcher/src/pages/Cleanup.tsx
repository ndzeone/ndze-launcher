import { useEffect, useState } from "react";
import { RefreshCw, Sparkles, Trash2 } from "lucide-react";
import { api, errorText, type CleanTarget } from "../api";
import { formatBytes } from "../format";
import { useToast } from "../toast";
import { PageHeader, Skeleton } from "../components/ui";

export function Cleanup() {
  const [targets, setTargets] = useState<CleanTarget[] | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set(["temp", "crash_dumps"]));
  const [running, setRunning] = useState(false);
  const notify = useToast();

  const scan = () => {
    setTargets(null);
    api.cleanupScan().then(setTargets).catch((e) => notify(errorText(e), "error"));
  };

  useEffect(scan, [notify]);

  const toggle = (id: string) =>
    setSelected((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const total = targets?.filter((t) => selected.has(t.id)).reduce((sum, t) => sum + t.size, 0) ?? 0;

  const clean = async () => {
    setRunning(true);
    try {
      const result = await api.cleanupRun([...selected]);
      notify(`Освобождено ${formatBytes(result.freed)}${result.skipped ? ` · пропущено занятых файлов: ${result.skipped}` : ""}`, "success");
      scan();
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setRunning(false);
    }
  };

  return (
    <section className="page">
      <PageHeader
        icon={Trash2}
        title="Очистка"
        subtitle="Освобождает место на диске. На FPS напрямую не влияет, но кэш шейдеров стоит сбрасывать после обновления драйвера, если появились статтеры."
        actions={
          <button className="btn" onClick={scan}>
            <RefreshCw size={15} /> Пересканировать
          </button>
        }
      />

      {!targets ? (
        <Skeleton height={300} />
      ) : (
        <>
          <div className="clean-list">
            {targets.map((t) => (
              <label key={t.id} className={`card clean ${selected.has(t.id) ? "selected" : ""}`}>
                <input type="checkbox" checked={selected.has(t.id)} onChange={() => toggle(t.id)} />
                <div className="grow">
                  <h3>{t.title}</h3>
                  <p>{t.description}</p>
                </div>
                <div className="clean-size">
                  <strong>{formatBytes(t.size)}</strong>
                  <span className="muted small">{t.files} файлов</span>
                </div>
              </label>
            ))}
          </div>
          <div className="clean-footer card">
            <div>
              <span className="muted small">Будет освобождено примерно</span>
              <strong className="big">{formatBytes(total)}</strong>
            </div>
            <button className="btn primary lg" onClick={clean} disabled={running || selected.size === 0}>
              <Sparkles size={17} />
              {running ? "Очищаю…" : "Очистить"}
            </button>
          </div>
        </>
      )}
    </section>
  );
}
