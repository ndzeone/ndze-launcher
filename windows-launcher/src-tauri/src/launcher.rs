use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Emitter};

use crate::error::{fail, AppResult};
use crate::power;
use crate::store::{BoostPlan, Game, GameSource, Priority, Settings};

const DETECT_TIMEOUT: Duration = Duration::from_secs(180);
const DETECT_INTERVAL: Duration = Duration::from_secs(2);
const EXIT_INTERVAL: Duration = Duration::from_secs(3);

/// Killing these would break the session; they are never allowed in a profile.
pub const PROTECTED: &[&str] = &[
    "system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "dwm.exe",
    "explorer.exe",
    "fontdrvhost.exe",
    "sihost.exe",
    "ctfmon.exe",
    "steam.exe",
    "steamwebhelper.exe",
    "epicgameslauncher.exe",
    "ndze-launcher.exe",
];

pub type ActiveSession = Arc<Mutex<Option<String>>>;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Starting,
    Running,
    Ended,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub game_id: String,
    pub phase: Phase,
    pub message: String,
}

fn emit(app: &AppHandle, game_id: &str, phase: Phase, message: impl Into<String>) {
    let _ = app.emit(
        "session",
        SessionEvent {
            game_id: game_id.to_owned(),
            phase,
            message: message.into(),
        },
    );
}

fn norm(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

fn processes_with_exe(sys: &mut System) {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
}

fn find_process(sys: &mut System, exe: &str) -> Option<Pid> {
    processes_with_exe(sys);
    let target = norm(exe);
    sys.processes()
        .iter()
        .find(|(_, p)| {
            p.exe()
                .is_some_and(|e| norm(&e.to_string_lossy()) == target)
        })
        .map(|(pid, _)| *pid)
}

fn close_processes(names: &[String], game_exe: Option<&str>) -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let game_name = game_exe
        .and_then(|e| std::path::Path::new(e).file_name())
        .map(|n| n.to_string_lossy().to_lowercase());
    let wanted: Vec<String> = names
        .iter()
        .map(|n| n.to_lowercase())
        .filter(|n| !PROTECTED.contains(&n.as_str()) && Some(n) != game_name.as_ref())
        .collect();

    let mut closed = Vec::new();
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy().to_lowercase();
        if wanted.contains(&name) && process.kill() && !closed.contains(&name) {
            closed.push(name);
        }
    }
    closed
}

fn start(game: &Game) -> AppResult<()> {
    match &game.source {
        GameSource::Steam { app_id } => open(&format!("steam://rungameid/{app_id}"), None, None),
        GameSource::Epic { launch_uri, .. } => open(launch_uri, None, None),
        GameSource::Manual => {
            let Some(exe) = &game.exe_path else {
                return fail("Не указан exe-файл игры");
            };
            let dir = std::path::Path::new(exe)
                .parent()
                .map(|d| d.display().to_string());
            let args = Some(game.profile.launch_args.trim()).filter(|a| !a.is_empty());
            open(exe, args, dir.as_deref())
        }
    }
}

#[cfg(windows)]
fn open(target: &str, args: Option<&str>, dir: Option<&str>) -> AppResult<()> {
    crate::win::shell_execute("open", target, args, dir, true, false).map(|_| ())
}

#[cfg(not(windows))]
fn open(target: &str, _: Option<&str>, _: Option<&str>) -> AppResult<()> {
    std::process::Command::new("xdg-open").arg(target).spawn()?;
    Ok(())
}

#[cfg(windows)]
fn set_priority(pid: Pid, priority: Priority) -> AppResult<()> {
    crate::win::set_priority(pid.as_u32(), priority)
}

#[cfg(not(windows))]
fn set_priority(_: Pid, _: Priority) -> AppResult<()> {
    fail("Приоритет процесса настраивается только в Windows")
}

fn boost_power(settings: &Settings) -> AppResult<String> {
    let previous = power::active()?;
    let target =
        match settings.boost_plan {
            BoostPlan::High => power::HIGH_PERFORMANCE.to_owned(),
            BoostPlan::Ultimate => match &settings.ultimate_guid {
                Some(g) => g.clone(),
                None => return fail(
                    "Сначала включите схему «Максимальная производительность» на вкладке «Система»",
                ),
            },
        };
    if previous != target {
        power::set_active(&target)?;
    }
    Ok(previous)
}

pub fn launch(
    app: AppHandle,
    session: ActiveSession,
    game: Game,
    settings: Settings,
) -> AppResult<()> {
    {
        let mut active = session.lock().unwrap();
        if let Some(id) = active.as_ref() {
            return fail(format!("Уже запущена игра: {id}"));
        }
        *active = Some(game.id.clone());
    }
    let release = |session: &ActiveSession| *session.lock().unwrap() = None;

    let profile = &game.profile;
    let exe = game.exe_path.clone();
    let mut notes = Vec::new();

    let restore_plan = if profile.boost_power_plan && exe.is_some() {
        match boost_power(&settings) {
            Ok(prev) => Some(prev),
            Err(e) => {
                notes.push(format!("Схема питания не переключена: {e}"));
                None
            }
        }
    } else {
        None
    };

    if !profile.close_processes.is_empty() {
        let closed = close_processes(&profile.close_processes, exe.as_deref());
        if !closed.is_empty() {
            notes.push(format!("Закрыто: {}", closed.join(", ")));
        }
    }

    if let Err(e) = start(&game) {
        if let Some(prev) = &restore_plan {
            let _ = power::set_active(prev);
        }
        release(&session);
        return Err(e);
    }
    emit(&app, &game.id, Phase::Starting, notes.join(". "));

    let Some(exe) = exe else {
        emit(&app, &game.id, Phase::Running, "Запущено без отслеживания: укажите exe в профиле, чтобы работали приоритет и схема питания");
        release(&session);
        return Ok(());
    };

    let priority = profile.priority;
    let id = game.id.clone();
    thread::spawn(move || {
        let mut sys = System::new();
        let started = Instant::now();
        let pid = loop {
            if let Some(pid) = find_process(&mut sys, &exe) {
                break Some(pid);
            }
            if started.elapsed() > DETECT_TIMEOUT {
                break None;
            }
            thread::sleep(DETECT_INTERVAL);
        };

        match pid {
            None => emit(
                &app,
                &id,
                Phase::Error,
                "Процесс игры не появился за 3 минуты — проверьте путь к exe в профиле",
            ),
            Some(pid) => {
                let mut message = format!("PID {pid}");
                if priority != Priority::Normal {
                    match set_priority(pid, priority) {
                        Ok(()) => message.push_str(", приоритет повышен"),
                        Err(e) => message.push_str(&format!(". {e}")),
                    }
                }
                emit(&app, &id, Phase::Running, message);

                loop {
                    thread::sleep(EXIT_INTERVAL);
                    processes_with_exe(&mut sys);
                    let alive = sys
                        .process(pid)
                        .and_then(|p| p.exe())
                        .is_some_and(|e| norm(&e.to_string_lossy()) == norm(&exe));
                    if !alive {
                        break;
                    }
                }
            }
        }

        let mut message = String::from("Игра закрыта");
        if let Some(prev) = restore_plan {
            match power::set_active(&prev) {
                Ok(()) => message.push_str(", схема питания восстановлена"),
                Err(e) => {
                    message.push_str(&format!(". Не удалось восстановить схему питания: {e}"))
                }
            }
        }
        emit(&app, &id, Phase::Ended, message);
        release(&session);
    });
    Ok(())
}
