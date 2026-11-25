import { useEffect, useState } from "react";

export const REPO = "ndzeone/ndze-launcher";
export const RELEASES_URL = `https://github.com/${REPO}/releases`;
export const LATEST_URL = `${RELEASES_URL}/latest`;

export interface Release {
  version: string;
  publishedAt: string;
  installer: { url: string; size: number } | null;
  msi: { url: string; size: number } | null;
  ipa: { url: string; size: number } | null;
}

interface GitHubAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

const pick = (assets: GitHubAsset[], suffix: string) => {
  const asset = assets.find((a) => a.name.toLowerCase().endsWith(suffix));
  return asset ? { url: asset.browser_download_url, size: asset.size } : null;
};

/** Reads the latest release from the public GitHub API; the page still works with plain links if it fails. */
export function useLatestRelease() {
  const [release, setRelease] = useState<Release | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      signal: controller.signal,
      headers: { Accept: "application/vnd.github+json" },
    })
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((data: { tag_name: string; published_at: string; assets: GitHubAsset[] }) =>
        setRelease({
          version: data.tag_name.replace(/^v/, ""),
          publishedAt: data.published_at,
          installer: pick(data.assets, "setup.exe"),
          msi: pick(data.assets, ".msi"),
          ipa: pick(data.assets, ".ipa"),
        }),
      )
      .catch(() => {});
    return () => controller.abort();
  }, []);
  return release;
}

export const formatSize = (bytes: number) => `${(bytes / 1024 / 1024).toFixed(1)} МБ`;
