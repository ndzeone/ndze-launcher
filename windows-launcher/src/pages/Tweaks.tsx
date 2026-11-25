import { useEffect, useState } from "react";
import { BatteryCharging, Copy, Plus, ShieldCheck, Trash2, Zap } from "lucide-react";
import { api, errorText, type PowerScheme, type TweakStatus } from "../api";
import { useToast } from "../toast";
import { Toggle } from "../components/Toggle";
import { PageHeader } from "../components/ui";

const IMPACT = { medium: "Заметный эффект", low: "Небольшой эффект" } as const;
const RESTART = { none: null, relogin: "Нужен перевход", reboot: "Нужна перезагрузка" } as const;

function PowerSchemes() {
  const [schemes, setSchemes] = useState<PowerScheme[]>([]);
  const notify = useToast();

  useEffect(() => {
    api.powerSchemes().then(setSchemes).catch((e) => notify(errorText(e), "error"));
  }, [notify]);

  const run = async (action: () => Promise<PowerScheme[]>, success?: string) => {
    try {
      setSchemes(await action());
      if (success) notify(success, "success");
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const dedupe = async () => {
    try {
      const [removed, list] = await api.dedupePowerSchemes();
      setSchemes(list);
      notify(removed ? `Удалено дубликатов: ${removed}` : "Дубликатов нет", "success");
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  const names = schemes.map((s) => s.name);
  const hasDuplicates = names.some((n, i) => names.indexOf(n) !== i);
  const hasUltimate = schemes.some((s) => s.kind === "ultimate");

  return (
    <>
      <div className="section-row">
        <h2 className="section-title">
          <BatteryCharging size={18} /> Схема питания
        </h2>
        <div className="actions">
          {hasDuplicates && (
            <button className="btn" onClick={dedupe}>
              <Copy size={15} /> Удалить дубликаты
            </button>
          )}
          {!hasUltimate && (
            <button className="btn" onClick={() => run(api.enableUltimate, "Схема «Максимальная производительность» добавлена")}>
              <Plus size={15} /> Максимальная производительность
            </button>
          )}
        </div>
      </div>
      <div className="schemes">
        {schemes.map((s) => (
          <div key={s.guid} className={`card scheme ${s.active ? "active" : ""}`}>
            <button className="scheme-main" onClick={() => !s.active && run(() => api.setPowerScheme(s.guid))}>
              <span className="radio" />
              <span className="grow">{s.name}</span>
              <span className={`tag kind-${s.kind}`}>{s.builtin ? "Windows" : s.kind === "custom" ? "Своя" : "Копия"}</span>
            </button>
            {!s.builtin && !s.active && (
              <button className="btn icon ghost" title="Удалить схему" aria-label={`Удалить ${s.name}`} onClick={() => run(() => api.deletePowerScheme(s.guid), "Схема удалена")}>
                <Trash2 size={15} />
              </button>
            )}
          </div>
        ))}
      </div>
    </>
  );
}

export function Tweaks() {
  const [tweaks, setTweaks] = useState<TweakStatus[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const notify = useToast();

  useEffect(() => {
    api.listTweaks().then(setTweaks).catch((e) => notify(errorText(e), "error"));
  }, [notify]);

  const toggle = async (tweak: TweakStatus, enabled: boolean) => {
    setBusy(tweak.id);
    try {
      setTweaks(await api.setTweak(tweak.id, enabled));
      const restart = RESTART[tweak.restart];
      notify(`${enabled ? "Применено" : "Отменено"}: ${tweak.title}${restart ? `. ${restart}` : ""}`, "success");
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setBusy(null);
    }
  };

  const applied = tweaks.filter((t) => t.applied).length;

  return (
    <section className="page">
      <PageHeader
        icon={Zap}
        title="Оптимизация Windows"
        subtitle={`Применено ${applied} из ${tweaks.length}. Исходные значения сохраняются — выключение возвращает всё как было.`}
      />

      <div className="tweaks">
        {tweaks.map((t) => (
          <article key={t.id} className={`card tweak ${t.applied ? "applied" : ""}`}>
            <div className="tweak-main">
              <h3>
                {t.title}
                {t.requiresAdmin && (
                  <span className="admin" title="Запросит права администратора (UAC)">
                    <ShieldCheck size={14} />
                  </span>
                )}
              </h3>
              <p>{t.description}</p>
              <div className="chips">
                <span className={`chip impact-${t.impact}`}>{IMPACT[t.impact]}</span>
                {RESTART[t.restart] && <span className="chip">{RESTART[t.restart]}</span>}
              </div>
            </div>
            <Toggle checked={t.applied} disabled={busy !== null} onChange={(v) => toggle(t, v)} label={t.title} />
          </article>
        ))}
      </div>

      <PowerSchemes />
    </section>
  );
}
