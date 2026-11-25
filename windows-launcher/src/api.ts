import { invoke } from "@tauri-apps/api/core";

export type Priority = "normal" | "aboveNormal" | "high";
export type BoostPlan = "high" | "ultimate";

export type GameSource =
  | { kind: "steam"; appId: string }
  | { kind: "epic"; appName: string; launchUri: string }
  | { kind: "manual" };

export interface GameProfile {
  highPerfGpu: boolean;
  disableFso: boolean;
  priority: Priority;
  boostPowerPlan: boolean;
  closeProcesses: string[];
  launchArgs: string;
}

export interface GameStats {
  playtimeSecs: number;
  launches: number;
  lastPlayed: number | null;
}

export interface Game {
  id: string;
  name: string;
  source: GameSource;
  installDir: string | null;
  exePath: string | null;
  profile: GameProfile;
  favorite: boolean;
  stats: GameStats;
}

export interface TweakStatus {
  id: string;
  title: string;
  description: string;
  impact: "medium" | "low";
  restart: "none" | "relogin" | "reboot";
  requiresAdmin: boolean;
  applied: boolean;
}

export interface PowerScheme {
  guid: string;
  name: string;
  kind: "balanced" | "high" | "saver" | "ultimate" | "custom";
  active: boolean;
  builtin: boolean;
}

export interface LiveStats {
  cpu: number;
  cores: number[];
  memoryUsed: number;
  memoryTotal: number;
}

export interface ProcessInfo {
  name: string;
  memory: number;
  count: number;
}

export interface ExeCandidate {
  path: string;
  size: number;
}

export interface Settings {
  boostPlan: BoostPlan;
  ultimateGuid: string | null;
  minimizeOnLaunch: boolean;
}

export interface DisplayInfo {
  device: string;
  name: string;
  width: number;
  height: number;
  refresh: number;
  maxRefresh: number;
  rates: number[];
  primary: boolean;
}

export interface HardwareReport {
  os: { name: string; version: string; build: string; arch: string };
  cpu: { name: string; vendor: string; cores: number; threads: number; baseMhz: number; l2Kb: number | null; l3Kb: number | null };
  gpus: { name: string; vendor: string; vram: number; driver: string | null }[];
  memory: {
    total: number;
    modules: { slot: string; capacity: number; speed: number; configuredSpeed: number; manufacturer: string; part: string; kind: string }[];
  };
  disks: { name: string; mount: string; kind: "ssd" | "hdd" | "unknown"; fileSystem: string; total: number; available: number; removable: boolean }[];
  displays: DisplayInfo[];
  board: { vendor: string; name: string } | null;
  laptop: boolean;
  elevated: boolean;
}

export type AdviceAction = { kind: "tweaks"; ids: string[] } | { kind: "refresh"; device: string; hz: number };

export interface Advice {
  overall: number;
  tier: string;
  verdictTitle: string;
  verdictText: string;
  scores: { key: "cpu" | "gpu" | "ram" | "storage"; label: string; value: number; caption: string }[];
  settings: { label: string; value: string; note: string }[];
  items: { id: string; tone: "good" | "tip" | "warn"; title: string; text: string; action: AdviceAction | null }[];
}

export interface StartupItem {
  name: string;
  command: string;
  machine: boolean;
  enabled: boolean;
}

export interface CleanTarget {
  id: string;
  title: string;
  description: string;
  size: number;
  files: number;
}

export interface SessionEvent {
  gameId: string;
  phase: "starting" | "running" | "ended" | "error";
  message: string;
}

export const api = {
  liveStats: () => invoke<LiveStats>("live_stats"),
  listProcesses: () => invoke<ProcessInfo[]>("list_processes"),

  listGames: () => invoke<Game[]>("list_games"),
  scanGames: () => invoke<Game[]>("scan_games"),
  addGame: (exePath: string, name?: string) => invoke<Game>("add_game", { exePath, name: name ?? null }),
  removeGame: (id: string) => invoke<void>("remove_game", { id }),
  updateGame: (id: string, exePath: string | null, profile: GameProfile) =>
    invoke<Game>("update_game", { id, exePath, profile }),
  exeCandidates: (id: string) => invoke<ExeCandidate[]>("exe_candidates", { id }),
  launchGame: (id: string) => invoke<void>("launch_game", { id }),
  toggleFavorite: (id: string) => invoke<Game>("toggle_favorite", { id }),
  activeSession: () => invoke<string | null>("active_session"),

  listTweaks: () => invoke<TweakStatus[]>("list_tweaks"),
  setTweak: (id: string, enabled: boolean) => invoke<TweakStatus[]>("set_tweak", { id, enabled }),
  applyTweaks: (ids: string[]) => invoke<TweakStatus[]>("apply_tweaks", { ids }),

  hardware: (refresh = false) => invoke<HardwareReport>("hardware_report", { refresh }),
  advice: () => invoke<Advice>("pc_advice"),
  setRefreshRate: (device: string, hz: number) => invoke<HardwareReport>("set_refresh_rate", { device, hz }),

  startupItems: () => invoke<StartupItem[]>("startup_items"),
  setStartup: (name: string, machine: boolean, enabled: boolean) =>
    invoke<StartupItem[]>("set_startup", { name, machine, enabled }),

  cleanupScan: () => invoke<CleanTarget[]>("cleanup_scan"),
  cleanupRun: (ids: string[]) => invoke<{ freed: number; skipped: number }>("cleanup_run", { ids }),

  powerSchemes: () => invoke<PowerScheme[]>("power_schemes"),
  setPowerScheme: (guid: string) => invoke<PowerScheme[]>("set_power_scheme", { guid }),
  enableUltimate: () => invoke<PowerScheme[]>("enable_ultimate"),
  deletePowerScheme: (guid: string) => invoke<PowerScheme[]>("delete_power_scheme", { guid }),
  dedupePowerSchemes: () => invoke<[number, PowerScheme[]]>("dedupe_power_schemes"),
  setMinimizeOnLaunch: (enabled: boolean) => invoke<Settings>("set_minimize_on_launch", { enabled }),
  getSettings: () => invoke<Settings>("get_settings"),
  setBoostPlan: (plan: BoostPlan) => invoke<Settings>("set_boost_plan", { plan }),
};

export const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
