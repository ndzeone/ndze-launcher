import { useEffect, useState } from "react";
import { CircleCheck, Cpu, Gauge, HardDrive, Lightbulb, MemoryStick, MonitorPlay, Sparkles, TriangleAlert, Wand2, type LucideIcon } from "lucide-react";
import { api, errorText, type Advice, type AdviceAction } from "../api";
import { useToast } from "../toast";
import { Meter, PageHeader, ScoreRing, Skeleton } from "../components/ui";
import type { Page } from "../App";

const SCORE_ICON: Record<string, LucideIcon> = { cpu: Cpu, gpu: MonitorPlay, ram: MemoryStick, storage: HardDrive };
const TONE_ICON = { good: CircleCheck, tip: Lightbulb, warn: TriangleAlert } as const;

export function Config({ onNavigate }: { onNavigate: (page: Page) => void }) {
  const [advice, setAdvice] = useState<Advice | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const notify = useToast();

  const load = () => api.advice().then(setAdvice).catch((e) => notify(errorText(e), "error"));

  useEffect(() => {
    load();
  }, []);

  const run = async (id: string, action: AdviceAction) => {
    setBusy(id);
    try {
      if (action.kind === "tweaks") {
        await api.applyTweaks(action.ids);
        notify("Рекомендованные настройки применены. Для HAGS нужна перезагрузка.", "success");
      } else {
        await api.setRefreshRate(action.device, action.hz);
        notify(`Монитор переключён на ${action.hz} Гц`, "success");
      }
      await load();
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setBusy(null);
    }
  };

  const actionLabel = (action: AdviceAction) => (action.kind === "tweaks" ? "Применить" : `Включить ${action.hz} Гц`);
  const pending = advice?.items.filter((i) => i.tone !== "good") ?? [];
  const done = advice?.items.filter((i) => i.tone === "good") ?? [];

  return (
    <section className="page">
      <PageHeader
        icon={Sparkles}
        title="Конфигурация под ваш ПК"
        subtitle="Оценка железа, узкое место и настройки, которые дадут больше FPS именно на этом компьютере."
      />

      {!advice ? (
        <>
          <Skeleton height={220} />
          <Skeleton height={300} />
        </>
      ) : (
        <>
          <section className="card verdict">
            <div className="verdict-score">
              <ScoreRing value={advice.overall} label="из 100" />
              <span className="tier">{advice.tier}</span>
            </div>
            <div className="verdict-body">
              <span className="eyebrow">Вердикт</span>
              <h2>{advice.verdictTitle}</h2>
              <p>{advice.verdictText}</p>
              <div className="scores">
                {advice.scores.map((s) => {
                  const Icon = SCORE_ICON[s.key] ?? Gauge;
                  return (
                    <div key={s.key} className="score">
                      <div className="score-head">
                        <Icon size={15} />
                        <span>{s.label}</span>
                        <strong>{s.value}</strong>
                      </div>
                      <Meter value={s.value} tone={s.value < 50 ? "warn" : undefined} />
                      <span className="muted small">{s.caption}</span>
                    </div>
                  );
                })}
              </div>
            </div>
          </section>

          <h2 className="section-title">
            <Wand2 size={18} /> Настройки графики в играх
          </h2>
          <div className="settings-grid">
            {advice.settings.map((s) => (
              <article key={s.label} className="card setting">
                <span className="muted small">{s.label}</span>
                <strong>{s.value}</strong>
                <p>{s.note}</p>
              </article>
            ))}
          </div>

          {pending.length > 0 && (
            <>
              <h2 className="section-title">
                <Lightbulb size={18} /> Что улучшить
              </h2>
              <div className="advice-list">
                {pending.map((item) => {
                  const Icon = TONE_ICON[item.tone];
                  return (
                    <article key={item.id} className={`card advice tone-${item.tone}`}>
                      <span className="advice-icon">
                        <Icon size={18} />
                      </span>
                      <div className="grow">
                        <h3>{item.title}</h3>
                        <p>{item.text}</p>
                      </div>
                      {item.action && (
                        <button className="btn primary" disabled={busy !== null} onClick={() => run(item.id, item.action!)}>
                          {busy === item.id ? "Применяю…" : actionLabel(item.action)}
                        </button>
                      )}
                    </article>
                  );
                })}
              </div>
            </>
          )}

          {done.length > 0 && (
            <>
              <h2 className="section-title">
                <CircleCheck size={18} /> Уже в порядке
              </h2>
              <div className="advice-done">
                {done.map((item) => (
                  <div key={item.id} className="done-item">
                    <CircleCheck size={16} />
                    <div>
                      <strong>{item.title}</strong>
                      <span className="muted small">{item.text}</span>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}

          <p className="hint">
            Оценки примерные: они строятся по модели CPU/GPU, объёму и режиму памяти, а не по бенчмаркам.{" "}
            <button className="link" onClick={() => onNavigate("pc")}>Подробности о железе →</button>
          </p>
        </>
      )}
    </section>
  );
}
