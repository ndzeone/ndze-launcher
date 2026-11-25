const GB = 1024 ** 3;
const MB = 1024 ** 2;

export const formatBytes = (bytes: number) =>
  bytes >= GB ? `${(bytes / GB).toFixed(bytes >= 100 * GB ? 0 : 1)} ГБ` : bytes >= MB ? `${Math.round(bytes / MB)} МБ` : `${Math.round(bytes / 1024)} КБ`;

export const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

export function formatDuration(secs: number) {
  if (secs < 60) return secs > 0 ? "< 1 мин" : "—";
  const hours = Math.floor(secs / 3600);
  const minutes = Math.floor((secs % 3600) / 60);
  return hours ? `${hours} ч ${minutes} мин` : `${minutes} мин`;
}

const rtf = new Intl.RelativeTimeFormat("ru", { numeric: "auto" });

export function relativeTime(unixSecs: number | null) {
  if (!unixSecs) return "ещё не запускалась";
  const diff = unixSecs - Date.now() / 1000;
  const abs = Math.abs(diff);
  if (abs < 60) return "только что";
  if (abs < 3600) return rtf.format(Math.round(diff / 60), "minute");
  if (abs < 86400) return rtf.format(Math.round(diff / 3600), "hour");
  if (abs < 86400 * 30) return rtf.format(Math.round(diff / 86400), "day");
  return rtf.format(Math.round(diff / (86400 * 30)), "month");
}

export const percent = (part: number, total: number) => (total > 0 ? Math.round((part / total) * 100) : 0);
