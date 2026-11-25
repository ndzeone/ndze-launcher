#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod games;
mod launcher;
mod power;
mod profile;
mod reg;
mod store;
mod system;
mod tweaks;
mod vdf;
#[cfg(windows)]
mod win;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use sysinfo::System;
use tauri::{AppHandle, Manager, State};

use error::{fail, AppResult};
use launcher::ActiveSession;
use store::{BoostPlan, Game, GameProfile, Settings, Store};

struct AppState {
    store: Mutex<Store>,
    stats: Mutex<System>,
    session: ActiveSession,
}

impl AppState {
    fn with_store<T>(&self, f: impl FnOnce(&mut Store) -> AppResult<T>) -> AppResult<T> {
        let mut store = self.store.lock().unwrap();
        f(&mut store)
    }
}

fn sorted_games(store: &Store) -> Vec<Game> {
    let mut games: Vec<Game> = store.config.games.values().cloned().collect();
    games.sort_by_key(|g| g.name.to_lowercase());
    games
}

#[tauri::command(async)]
fn system_info() -> system::SystemInfo {
    system::info()
}

#[tauri::command]
fn live_stats(state: State<AppState>) -> system::LiveStats {
    system::live(&mut state.stats.lock().unwrap())
}

#[tauri::command(async)]
fn list_processes() -> Vec<system::ProcessInfo> {
    system::processes()
}

#[tauri::command]
fn list_games(state: State<AppState>) -> Vec<Game> {
    sorted_games(&state.store.lock().unwrap())
}

#[tauri::command(async)]
fn scan_games(state: State<AppState>) -> AppResult<Vec<Game>> {
    let mut detected = games::scan_steam();
    detected.extend(games::scan_epic());

    state.with_store(|store| {
        let games = &mut store.config.games;
        games.retain(|id, g| {
            matches!(g.source, store::GameSource::Manual) || detected.iter().any(|d| &d.id == id)
        });
        for mut found in detected {
            if let Some(existing) = games.get(&found.id) {
                found.profile = existing.profile.clone();
                if existing.exe_path.is_some() {
                    found.exe_path = existing.exe_path.clone();
                }
            }
            games.insert(found.id.clone(), found);
        }
        store.save()?;
        Ok(sorted_games(store))
    })
}

#[tauri::command(async)]
fn add_game(state: State<AppState>, exe_path: String, name: Option<String>) -> AppResult<Game> {
    let exe = PathBuf::from(&exe_path);
    if !exe.is_file() || !exe_path.to_lowercase().ends_with(".exe") {
        return fail("Выберите существующий .exe файл");
    }
    let game = games::manual(&exe, name);
    state.with_store(|store| {
        store.config.games.insert(game.id.clone(), game.clone());
        store.save()?;
        Ok(game)
    })
}

#[tauri::command(async)]
fn remove_game(state: State<AppState>, id: String) -> AppResult<()> {
    state.with_store(|store| {
        if let Some(game) = store.config.games.remove(&id) {
            if let Some(exe) = &game.exe_path {
                profile::apply(exe, false, false)?;
            }
        }
        store.save()
    })
}

#[tauri::command(async)]
fn update_game(
    state: State<AppState>,
    id: String,
    exe_path: Option<String>,
    profile: GameProfile,
) -> AppResult<Game> {
    if let Some(bad) = profile
        .close_processes
        .iter()
        .find(|p| launcher::PROTECTED.contains(&p.to_lowercase().as_str()))
    {
        return fail(format!("{bad} — системный процесс, его нельзя закрывать"));
    }
    let exe_path = exe_path.filter(|p| !p.trim().is_empty());
    if let Some(exe) = &exe_path {
        if !PathBuf::from(exe).is_file() {
            return fail(format!("Файл не найден: {exe}"));
        }
    }

    state.with_store(|store| {
        let Some(game) = store.config.games.get_mut(&id) else {
            return fail("Игра не найдена");
        };
        if let Some(old) = game
            .exe_path
            .as_ref()
            .filter(|old| Some(*old) != exe_path.as_ref())
        {
            profile::apply(old, false, false)?;
        }
        if let Some(exe) = &exe_path {
            profile::apply(exe, profile.high_perf_gpu, profile.disable_fso)?;
        }
        game.exe_path = exe_path;
        game.profile = profile;
        let updated = game.clone();
        store.save()?;
        Ok(updated)
    })
}

#[tauri::command(async)]
fn exe_candidates(state: State<AppState>, id: String) -> AppResult<Vec<games::ExeCandidate>> {
    let dir = state.with_store(|store| {
        Ok(store
            .config
            .games
            .get(&id)
            .and_then(|g| g.install_dir.clone()))
    })?;
    Ok(dir
        .map(|d| games::exe_candidates(&PathBuf::from(d)))
        .unwrap_or_default())
}

#[tauri::command(async)]
fn launch_game(app: AppHandle, state: State<AppState>, id: String) -> AppResult<()> {
    let (game, settings) = state.with_store(|store| {
        let game = store.config.games.get(&id).cloned();
        Ok((game, store.config.settings.clone()))
    })?;
    let Some(game) = game else {
        return fail("Игра не найдена");
    };
    launcher::launch(app, state.session.clone(), game, settings)
}

#[tauri::command]
fn active_session(state: State<AppState>) -> Option<String> {
    state.session.lock().unwrap().clone()
}

#[tauri::command(async)]
fn list_tweaks() -> AppResult<Vec<tweaks::TweakStatus>> {
    tweaks::list()
}

#[tauri::command(async)]
fn set_tweak(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> AppResult<Vec<tweaks::TweakStatus>> {
    state.with_store(|store| {
        let result = if enabled {
            tweaks::apply(&id, &mut store.config)
        } else {
            tweaks::revert(&id, &mut store.config)
        };
        store.save()?;
        result
    })?;
    tweaks::list()
}

#[tauri::command(async)]
fn power_schemes(state: State<AppState>) -> AppResult<Vec<power::PowerScheme>> {
    let ultimate = state.with_store(|s| Ok(s.config.settings.ultimate_guid.clone()))?;
    power::list(ultimate.as_deref())
}

#[tauri::command(async)]
fn set_power_scheme(state: State<AppState>, guid: String) -> AppResult<Vec<power::PowerScheme>> {
    power::set_active(&guid)?;
    power_schemes(state)
}

#[tauri::command(async)]
fn enable_ultimate(state: State<AppState>) -> AppResult<Vec<power::PowerScheme>> {
    state.with_store(|store| {
        let guid = power::ensure_ultimate(store.config.settings.ultimate_guid.as_deref())?;
        store.config.settings.ultimate_guid = Some(guid);
        store.save()
    })?;
    power_schemes(state)
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.store.lock().unwrap().config.settings.clone()
}

#[tauri::command(async)]
fn set_boost_plan(state: State<AppState>, plan: BoostPlan) -> AppResult<Settings> {
    state.with_store(|store| {
        store.config.settings.boost_plan = plan;
        store.save()?;
        Ok(store.config.settings.clone())
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("config.json");
            app.manage(AppState {
                store: Mutex::new(Store::load(path)),
                stats: Mutex::new(system::stats_system()),
                session: Arc::new(Mutex::new(None)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            system_info,
            live_stats,
            list_processes,
            list_games,
            scan_games,
            add_game,
            remove_game,
            update_game,
            exe_candidates,
            launch_game,
            active_session,
            list_tweaks,
            set_tweak,
            power_schemes,
            set_power_scheme,
            enable_ultimate,
            get_settings,
            set_boost_plan,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start NDZE Launcher");
}
