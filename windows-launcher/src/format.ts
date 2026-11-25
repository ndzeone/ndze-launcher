const GB = 1024 ** 3;
const MB = 1024 ** 2;

export const formatBytes = (bytes: number) =>
  bytes >= GB ? `${(bytes / GB).toFixed(1)} ГБ` : `${Math.round(bytes / MB)} МБ`;

export const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;
