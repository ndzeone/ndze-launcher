import { DesktopTools } from "../components/DesktopTools";
import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, Globe, Info, Settings as SettingsIcon, Smartphone, Zap } from "lucide-react";
import { api, errorText, type BoostPlan, type Settings as AppSettings } from "../api";
import { useToast } from "../toast";
import { Toggle } from "../components/Toggle";
import { PageHeader } from "../components/ui";

const SITE = "https://ndzeone.github.io/ndze-launcher/";
const RELEASES = "https://github.com/ndzeone/ndze-launcher/releases";

export function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [version, setVersion] = useState("");
  const notify = useToast();

  useEffect(() => {
    api.getSettings().then(setSettings).catch((e) => notify(errorText(e), "error"));
    getVersion().then(setVersion).catch(() => {});
  }, [notify]);

  const update = async (action: () => Promise<AppSettings>) => {
    try {
      setSettings(await action());
    } catch (e) {
      notify(errorText(e), "error");
    }
  };

  return (
    <section className="page narrow">
      <PageHeader icon={SettingsIcon} title="Настройки" />
      <DesktopTools />

      <div className="card list">
        <div className="list-row">
          <div className="grow list-text">
            <strong>Сворачивать лаунчер при запуске игры</strong>
            <span className="muted small">Окно уходит в панель задач и не мешает игре. Отслеживание процесса продолжает работать.</span>
          </div>
          <Toggle
            checked={settings?.minimizeOnLaunch ?? false}
            onChange={(v) => update(() => api.setMinimizeOnLaunch(v))}
            label="Сворачивать при запуске"
          />
        </div>
        <div className="list-row">
          <div className="grow list-text">
            <strong>Схема питания во время игры</strong>
            <span className="muted small">Для игр с опцией «Схема питания на время игры» в профиле.</span>
          </div>
          <div className="segmented">
            {(["high", "ultimate"] as BoostPlan[]).map((plan) => (
              <button
                key={plan}
                className={settings?.boostPlan === plan ? "active" : ""}
                onClick={() => update(() => api.setBoostPlan(plan))}
                disabled={plan === "ultimate" && !settings?.ultimateGuid}
                title={plan === "ultimate" && !settings?.ultimateGuid ? "Добавьте схему на вкладке «Оптимизация»" : undefined}
              >
                {plan === "high" ? "Высокая" : "Максимальная"}
              </button>
            ))}
          </div>
        </div>
      </div>

      <h2 className="section-title">
        <Info size={18} /> О программе
      </h2>
      <div className="card about">
        <div className="about-brand">
          <span className="brand-mark lg">
            <Zap size={26} fill="currentColor" strokeWidth={0} />
          </span>
          <div>
            <strong>NDZE Launcher {version && `v${version}`}</strong>
            <span className="muted small">Tauri 2 · Rust · React · TypeScript</span>
          </div>
        </div>
        <div className="about-links">
          <button className="btn" onClick={() => openUrl(SITE)}>
            <Globe size={15} /> Сайт <ExternalLink size={13} />
          </button>
          <button className="btn" onClick={() => openUrl(RELEASES)}>
            Все версии <ExternalLink size={13} />
          </button>
          <button className="btn" onClick={() => openUrl(`${SITE}#ios`)}>
            <Smartphone size={15} /> NDZE AI для iPhone <ExternalLink size={13} />
          </button>
        </div>
      </div>
    </section>
  );
}
