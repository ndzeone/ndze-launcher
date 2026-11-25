import { useEffect, useState, type ReactNode } from "react";
import { type LucideIcon, AppWindow, CircuitBoard, Cpu, HardDrive, MemoryStick, Monitor, MonitorPlay, RefreshCw, Sparkles } from "lucide-react";
import { api, errorText, type HardwareReport, type LiveStats } from "../api";
import { formatBytes, percent } from "../format";
import { useToast } from "../toast";
import { Meter, PageHeader, Skeleton, Spec } from "../components/ui";
import type { Page } from "../App";

function gpuTech(name: string, vendor: string) {
  const n = name.toLowerCase();
  const rtx = n.match(/rtx\s*(\d{2})\d{2}/);
  if (rtx) {
    const tech = ["DLSS", "Reflex", "Ray Tracing"];
    if (Number(rtx[1]) >= 40) tech.splice(1, 0, "Frame Generation");
    return tech;
  }
  if (vendor === "NVIDIA") return ["Reflex", "FSR", "NVENC"];
  if (vendor === "AMD") return ["FSR 3", "Anti-Lag", "Radeon Chill"];
  if (vendor === "Intel") return ["XeSS", "FSR"];
  return [];
}

const ghz = (mhz: number) => (mhz ? `${(mhz / 1000).toFixed(2)} ГГц` : "—");
const cleanCpu = (name: string) => name.replace(/\((R|TM)\)/g, "").replace(/\s+/g, " ").trim();

function Block({ icon: Icon, title, children, wide }: { icon: LucideIcon; title: string; children: ReactNode; wide?: boolean }) {
  return (
    <article className={`card block ${wide ? "wide" : ""}`}>
      <header className="block-head">
        <span className="block-icon">
          <Icon size={18} />
        </span>
        <h3>{title}</h3>
      </header>
      {children}
    </article>
  );
}

export function MyPC({ onNavigate }: { onNavigate: (page: Page) => void }) {
  const [report, setReport] = useState<HardwareReport | null>(null);
  const [live, setLive] = useState<LiveStats | null>(null);
  const [loading, setLoading] = useState(false);
  const notify = useToast();

  const load = async (refresh: boolean) => {
    setLoading(true);
    try {
      setReport(await api.hardware(refresh));
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load(false);
    const tick = () => api.liveStats().then(setLive).catch(() => {});
    tick();
    const timer = setInterval(tick, 1500);
    return () => clearInterval(timer);
  }, []);

  const changeRefresh = async (device: string, hz: number) => {
    try {
      setReport(await api.setRefreshRate(device, hz));
      notify(`Частота монитора: ${hz} Гц`, "success");
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const memoryUsed = live ? percent(live.memoryUsed, live.memoryTotal) : 0;

  return (
    <section className="page">
      <PageHeader
        icon={Cpu}
        title="Мой ПК"
        subtitle={report ? `${report.os.name}${report.os.build ? ` · сборка ${report.os.build}` : ""}` : "Собираю сведения о железе…"}
        actions={
          <>
            <button className="btn" onClick={() => load(true)} disabled={loading}>
              <RefreshCw size={15} className={loading ? "spin" : ""} />
              Обновить
            </button>
            <button className="btn primary" onClick={() => onNavigate("config")}>
              <Sparkles size={15} />
              Конфигурация под мой ПК
            </button>
          </>
        }
      />

      {!report ? (
        <div className="blocks">
          {Array.from({ length: 6 }, (_, i) => <Skeleton key={i} height={220} />)}
        </div>
      ) : (
        <div className="blocks">
          <Block icon={Cpu} title="Процессор">
            <h4 className="block-title">{cleanCpu(report.cpu.name)}</h4>
            <div className="specs">
              <Spec label="Ядра / потоки" value={`${report.cpu.cores} / ${report.cpu.threads}`} />
              <Spec label="Базовая частота" value={ghz(report.cpu.baseMhz)} />
              {report.cpu.l2Kb && <Spec label="Кэш L2" value={formatBytes(report.cpu.l2Kb * 1024)} />}
              {report.cpu.l3Kb && <Spec label="Кэш L3" value={formatBytes(report.cpu.l3Kb * 1024)} />}
            </div>
            <div className="live-row">
              <span>Загрузка</span>
              <strong>{live ? `${live.cpu.toFixed(0)}%` : "—"}</strong>
            </div>
            <div className="cores">
              {(live?.cores ?? Array(report.cpu.threads).fill(0)).map((v, i) => (
                <span key={i} className="core" title={`Поток ${i + 1}: ${v.toFixed(0)}%`}>
                  <span style={{ height: `${Math.max(4, v)}%` }} />
                </span>
              ))}
            </div>
          </Block>

          <Block icon={MonitorPlay} title="Видеокарта">
            {report.gpus.length === 0 && <p className="muted">Видеокарта не определена</p>}
            {report.gpus.map((gpu) => (
              <div key={gpu.name} className="gpu-item">
                <h4 className="block-title">{gpu.name}</h4>
                <div className="specs">
                  <Spec label="Производитель" value={gpu.vendor || "—"} />
                  <Spec label="Видеопамять" value={formatBytes(gpu.vram)} />
                  <Spec label="Драйвер" value={gpu.driver ?? "—"} />
                </div>
                {gpuTech(gpu.name, gpu.vendor).length > 0 && (
                  <div className="tech">
                    <span className="muted small">Поддерживает</span>
                    <div className="chips">
                      {gpuTech(gpu.name, gpu.vendor).map((t) => (
                        <span key={t} className="chip accent">{t}</span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </Block>

          <Block icon={MemoryStick} title="Оперативная память">
            <h4 className="block-title">
              {formatBytes(report.memory.total)}
              {report.memory.modules[0]?.kind && ` ${report.memory.modules[0].kind}`}
              {report.memory.modules.length > 0 && (
                <span className={`tag ${report.memory.modules.length >= 2 ? "good" : "warn"}`}>
                  {report.memory.modules.length >= 2 ? "Двухканал" : "Одноканал"}
                </span>
              )}
            </h4>
            <div className="live-row">
              <span>Занято {live ? formatBytes(live.memoryUsed) : "—"}</span>
              <strong>{memoryUsed}%</strong>
            </div>
            <Meter value={memoryUsed} />
            <div className="modules">
              {report.memory.modules.map((m, i) => (
                <div key={`${m.slot}-${i}`} className="module">
                  <span className="module-slot">{m.slot || `Слот ${i + 1}`}</span>
                  <span>{formatBytes(m.capacity)}</span>
                  <span>{m.configuredSpeed || m.speed} МГц</span>
                  <span className="muted small">{m.manufacturer || m.part}</span>
                </div>
              ))}
            </div>
          </Block>

          <Block icon={HardDrive} title="Накопители">
            <div className="disks">
              {report.disks.map((d) => {
                const used = percent(d.total - d.available, d.total);
                return (
                  <div key={d.mount} className="disk">
                    <div className="disk-head">
                      <strong>{d.mount.replace(/\\$/, "")}</strong>
                      <span className={`tag ${d.kind === "ssd" ? "good" : d.kind === "hdd" ? "warn" : ""}`}>{d.kind === "unknown" ? "—" : d.kind.toUpperCase()}</span>
                      <span className="muted small grow">{d.name}</span>
                      <span className="small">{formatBytes(d.available)} свободно из {formatBytes(d.total)}</span>
                    </div>
                    <Meter value={used} />
                  </div>
                );
              })}
            </div>
          </Block>

          <Block icon={Monitor} title="Мониторы">
            {report.displays.length === 0 && <p className="muted">Мониторы не определены</p>}
            {report.displays.map((d) => (
              <div key={d.device} className="display">
                <div className="display-head">
                  <h4 className="block-title">{d.name}</h4>
                  {d.primary && <span className="tag">Основной</span>}
                </div>
                <div className="specs">
                  <Spec label="Разрешение" value={`${d.width}×${d.height}`} />
                  <Spec label="Частота" value={`${d.refresh} Гц`} />
                  <Spec label="Максимум" value={`${d.maxRefresh} Гц`} />
                </div>
                {d.rates.length > 1 && (
                  <div className="rates">
                    {d.rates.filter((r) => r >= 50).map((r) => (
                      <button key={r} className={`rate ${r === d.refresh ? "active" : ""}`} onClick={() => r !== d.refresh && changeRefresh(d.device, r)}>
                        {r} Гц
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </Block>

          <Block icon={CircuitBoard} title="Система">
            <div className="specs">
              <Spec label="Материнская плата" value={report.board ? `${report.board.vendor} ${report.board.name}` : "—"} />
              <Spec label="ОС" value={report.os.name || "—"} />
              <Spec label="Версия" value={report.os.version || "—"} />
              <Spec label="Разрядность" value={report.os.arch || "—"} />
              <Spec label="Тип" value={report.laptop ? "Ноутбук" : "Настольный ПК"} />
              <Spec label="Права" value={report.elevated ? "Администратор" : "Пользователь (UAC по запросу)"} />
            </div>
            <p className="hint">
              <AppWindow size={13} /> Сведения берутся из WMI и DXGI — без установки драйверов и сторонних утилит.
            </p>
          </Block>
        </div>
      )}
    </section>
  );
}
