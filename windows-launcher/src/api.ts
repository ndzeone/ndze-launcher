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

export interface Game {
  id: string;
  name: string;
  source: GameSource;
  installDir: string | null;
  exePath: string | null;
  profile: GameProfile;
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
  active: boolean;
}

export interface GpuInfo {
  name: string;
  vram: number;
}

export interface SystemInfo {
  os: string;
  cpu: string;
  physicalCores: number | null;
  threads: number;
  totalMemory: number;
  gpus: GpuInfo[];
  elevated: boolean;
}

export interface LiveStats {
  cpu: number;
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
}

export interface SessionEvent {
  gameId: string;
  phase: "starting" | "running" | "ended" | "error";
  message: string;
}

export const api = {
  systemInfo: () => invoke<SystemInfo>("system_info"),
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
  activeSession: () => invoke<string | null>("active_session"),

  listTweaks: () => invoke<TweakStatus[]>("list_tweaks"),
  setTweak: (id: string, enabled: boolean) => invoke<TweakStatus[]>("set_tweak", { id, enabled }),

  powerSchemes: () => invoke<PowerScheme[]>("power_schemes"),
  setPowerScheme: (guid: string) => invoke<PowerScheme[]>("set_power_scheme", { guid }),
  enableUltimate: () => invoke<PowerScheme[]>("enable_ultimate"),
  getSettings: () => invoke<Settings>("get_settings"),
  setBoostPlan: (plan: BoostPlan) => invoke<Settings>("set_boost_plan", { plan }),
};

export const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
