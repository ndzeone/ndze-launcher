import { useEffect, useState } from "react";
import { api, errorText, type BoostPlan, type PowerScheme, type Settings, type SystemInfo } from "../api";
import { formatBytes } from "../format";
import { useToast } from "../toast";

const HISTORY = 60;

function Sparkline({ values }: { values: number[] }) {
  const offset = HISTORY - values.length;
  const points = values.map((v, i) => `${((offset + i) / (HISTORY - 1)) * 100},${30 - (v / 100) * 30}`).join(" ");
  return (
    <svg className="spark" viewBox="0 0 100 30" preserveAspectRatio="none" aria-hidden>
      <polyline points={points} />
    </svg>
  );
}

function Meter({ label, value, detail }: { label: string; value: number; detail: string }) {
  return (
    <div className="meter">
      <div className="meter-head">
        <span>{label}</span>
        <strong>{detail}</strong>
      </div>
      <div className="bar">
        <span style={{ width: `${Math.min(100, value)}%` }} />
      </div>
    </div>
  );
}

export function SystemPage() {
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [cpu, setCpu] = useState<number[]>([]);
  const [memory, setMemory] = useState({ used: 0, total: 1 });
  const [schemes, setSchemes] = useState<PowerScheme[]>([]);
  const [settings, setSettings] = useState<Settings | null>(null);
  const notify = useToast();

  useEffect(() => {
    api.systemInfo().then(setInfo).catch((e) => notify(errorText(e), "error"));
    api.powerSchemes().then(setSchemes).catch((e) => notify(errorText(e), "error"));
    api.getSettings().then(setSettings).catch(() => {});

    const tick = () =>
      api
        .liveStats()
        .then((s) => {
          setCpu((h) => [...h, s.cpu].slice(-HISTORY));
          setMemory({ used: s.memoryUsed, total: s.memoryTotal || 1 });
        })
        .catch(() => {});
    tick();
    const timer = setInterval(tick, 1500);
    return () => clearInterval(timer);
  }, [notify]);

  const run = async (action: () => Promise<PowerScheme[]>) => {
    try {
      setSchemes(await action());
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const enableUltimate = async () => {
    await run(api.enableUltimate);
    api.getSettings().then(setSettings).catch(() => {});
  };

  const setBoost = async (plan: BoostPlan) => {
    try {
      setSettings(await api.setBoostPlan(plan));
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const currentCpu = cpu[cpu.length - 1] ?? 0;
  const hasUltimate = !!settings?.ultimateGuid && schemes.some((s) => s.guid === settings.ultimateGuid);

  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>Система</h1>
          <p className="muted">{info?.os ?? "Загрузка…"}</p>
        </div>
      </header>

      <div className="system-grid">
        <article className="card stat">
          <span className="muted small">Процессор</span>
          <h3>{info?.cpu || "—"}</h3>
          <p className="muted small">
            {info ? `${info.physicalCores ?? "?"} ядер · ${info.threads} потоков` : ""}
          </p>
          <Meter label="Загрузка" value={currentCpu} detail={`${currentCpu.toFixed(0)}%`} />
          <Sparkline values={cpu} />
        </article>

        <article className="card stat">
          <span className="muted small">Память</span>
          <h3>{info ? formatBytes(info.totalMemory) : "—"}</h3>
          <p className="muted small">Оперативная память</p>
          <Meter
            label="Занято"
            value={(memory.used / memory.total) * 100}
            detail={`${formatBytes(memory.used)} / ${formatBytes(memory.total)}`}
          />
        </article>

        <article className="card stat wide">
          <span className="muted small">Видеокарты</span>
          {info?.gpus.length ? (
            info.gpus.map((g) => (
              <div key={g.name} className="gpu">
                <h3>{g.name}</h3>
                <span className="chip">{formatBytes(g.vram)} VRAM</span>
              </div>
            ))
          ) : (
            <h3>—</h3>
          )}
        </article>
      </div>

      <h2 className="section-title">Схема питания</h2>
      <div className="schemes">
        {schemes.map((s) => (
          <button
            key={s.guid}
            className={`card scheme ${s.active ? "active" : ""}`}
            onClick={() => !s.active && run(() => api.setPowerScheme(s.guid))}
          >
            <span className="radio" />
            {s.name}
          </button>
        ))}
        {!hasUltimate && (
          <button className="card scheme add" onClick={enableUltimate}>
            + Включить «Максимальную производительность»
          </button>
        )}
      </div>

      <h2 className="section-title">Во время игры использовать</h2>
      <div className="segmented">
        {(["high", "ultimate"] as const).map((plan) => (
          <button
            key={plan}
            className={settings?.boostPlan === plan ? "active" : ""}
            onClick={() => setBoost(plan)}
            disabled={plan === "ultimate" && !hasUltimate}
          >
            {plan === "high" ? "Высокая производительность" : "Максимальная производительность"}
          </button>
        ))}
      </div>
      <p className="hint">
        Применяется для игр с включённой опцией «Схема питания на время игры». «Максимальная» убирает микрозадержки
        энергосбережения CPU, но заметно повышает потребление — на ноутбуке от батареи не рекомендуется.
      </p>
    </section>
  );
}
