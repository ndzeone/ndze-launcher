import { useEffect, useState } from "react";
import { api, errorText, type TweakStatus } from "../api";
import { useToast } from "../toast";
import { Toggle } from "../components/Toggle";

const IMPACT = { medium: "Заметный эффект", low: "Небольшой эффект" } as const;
const RESTART = { none: null, relogin: "Нужен перевход", reboot: "Нужна перезагрузка" } as const;

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

  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>Оптимизация Windows</h1>
          <p className="muted">
            Только проверенные настройки, без «чистильщиков RAM». Перед изменением сохраняется исходное значение — выключение
            переключателя возвращает всё как было. Пункты со щитом запросят права администратора (UAC).
          </p>
        </div>
      </header>

      <div className="tweaks">
        {tweaks.map((t) => (
          <article key={t.id} className={`card tweak ${t.applied ? "applied" : ""}`}>
            <div className="tweak-main">
              <h3>
                {t.title}
                {t.requiresAdmin && (
                  <svg className="shield" viewBox="0 0 24 24" aria-label="Требуются права администратора">
                    <path d="M12 2 4 5v6c0 5 3.4 9.7 8 11 4.6-1.3 8-6 8-11V5z" />
                  </svg>
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
    </section>
  );
}
