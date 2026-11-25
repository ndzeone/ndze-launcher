import { useEffect, useMemo, useState } from "react";
import { Rocket, Search, ShieldCheck } from "lucide-react";
import { api, errorText, type StartupItem } from "../api";
import { useToast } from "../toast";
import { Toggle } from "../components/Toggle";
import { PageHeader, Skeleton } from "../components/ui";

const exePath = (command: string) => {
  const quoted = command.match(/^"([^"]+)"/);
  return quoted?.[1] ?? command.split(" ")[0] ?? command;
};

export function Startup() {
  const [items, setItems] = useState<StartupItem[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const notify = useToast();

  useEffect(() => {
    api.startupItems().then(setItems).catch((e) => notify(errorText(e), "error"));
  }, [notify]);

  const toggle = async (item: StartupItem, enabled: boolean) => {
    setBusy(item.name);
    try {
      setItems(await api.setStartup(item.name, item.machine, enabled));
      notify(`${item.name}: ${enabled ? "включено" : "отключено"} в автозагрузке`, "success");
    } catch (e) {
      notify(errorText(e), "error");
    } finally {
      setBusy(null);
    }
  };

  const visible = useMemo(
    () => (items ?? []).filter((i) => i.name.toLowerCase().includes(query.trim().toLowerCase())),
    [items, query],
  );
  const enabled = items?.filter((i) => i.enabled).length ?? 0;

  return (
    <section className="page">
      <PageHeader
        icon={Rocket}
        title="Автозагрузка"
        subtitle={items ? `Запускается вместе с Windows: ${enabled} из ${items.length}. Меньше программ — быстрее старт и больше свободной памяти в играх.` : "Загрузка…"}
        actions={
          <label className="search">
            <Search size={15} />
            <input placeholder="Поиск…" value={query} onChange={(e) => setQuery(e.target.value)} />
          </label>
        }
      />
      <p className="hint">
        Работает как «Диспетчер задач → Автозагрузка»: программа не удаляется, только перестаёт запускаться сама. Пункты со щитом относятся ко всем
        пользователям и запросят права администратора.
      </p>

      {!items ? (
        <Skeleton height={260} />
      ) : (
        <div className="card list">
          {visible.map((item) => (
            <div key={`${item.machine}-${item.name}`} className={`list-row ${item.enabled ? "" : "off"}`}>
              <span className="app-avatar">{item.name.slice(0, 1).toUpperCase()}</span>
              <div className="grow list-text">
                <strong>
                  {item.name}
                  {item.machine && (
                    <span className="admin" title="Для всех пользователей — нужен UAC">
                      <ShieldCheck size={13} />
                    </span>
                  )}
                </strong>
                <span className="muted small mono" title={item.command}>{exePath(item.command)}</span>
              </div>
              <Toggle checked={item.enabled} disabled={busy !== null} onChange={(v) => toggle(item, v)} label={item.name} />
            </div>
          ))}
          {visible.length === 0 && <div className="empty slim">Ничего не найдено</div>}
        </div>
      )}
    </section>
  );
}
