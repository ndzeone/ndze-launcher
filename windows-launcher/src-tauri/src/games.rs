use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::store::{Game, GameProfile, GameSource};
use crate::vdf;

const STEAM_REDIST_APPID: &str = "228980";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExeCandidate {
    pub path: String,
    pub size: u64,
}

fn game(
    id: String,
    name: String,
    source: GameSource,
    dir: Option<PathBuf>,
    exe: Option<PathBuf>,
) -> Game {
    Game {
        id,
        name,
        source,
        install_dir: dir.map(|d| d.display().to_string()),
        exe_path: exe.map(|e| e.display().to_string()),
        profile: GameProfile::default(),
    }
}

#[cfg(windows)]
fn steam_root() -> Option<PathBuf> {
    use crate::reg::{read_str, Hive};
    read_str(Hive::Hkcu, r"Software\Valve\Steam", "SteamPath")
        .ok()
        .flatten()
        .or_else(|| {
            read_str(
                Hive::Hklm,
                r"SOFTWARE\WOW6432Node\Valve\Steam",
                "InstallPath",
            )
            .ok()
            .flatten()
        })
        .map(|p| PathBuf::from(p.replace('/', "\\")))
}

#[cfg(not(windows))]
fn steam_root() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    [".local/share/Steam", ".steam/steam"]
        .iter()
        .map(|p| home.join(p))
        .find(|p| p.is_dir())
}

fn steam_libraries(root: &Path) -> Vec<PathBuf> {
    let mut libs = vec![root.to_path_buf()];
    if let Ok(src) = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")) {
        let doc = vdf::parse(&src);
        if let Some(folders) = doc.get("libraryfolders") {
            for (_, folder) in folders.entries() {
                if let Some(path) = folder.str("path") {
                    let path = PathBuf::from(path);
                    if !libs.iter().any(|l| same_path(l, &path)) {
                        libs.push(path);
                    }
                }
            }
        }
    }
    libs
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .eq_ignore_ascii_case(b.to_string_lossy().trim_end_matches(['\\', '/']))
}

pub fn scan_steam() -> Vec<Game> {
    let Some(root) = steam_root() else {
        return Vec::new();
    };
    let mut games = Vec::new();
    for lib in steam_libraries(&root) {
        let apps = lib.join("steamapps");
        let Ok(dir) = std::fs::read_dir(&apps) else {
            continue;
        };
        for entry in dir.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            if !(file.starts_with("appmanifest_") && file.ends_with(".acf")) {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let doc = vdf::parse(&src);
            let Some(state) = doc.get("AppState") else {
                continue;
            };
            let (Some(app_id), Some(name), Some(install)) = (
                state.str("appid"),
                state.str("name"),
                state.str("installdir"),
            ) else {
                continue;
            };
            if app_id == STEAM_REDIST_APPID
                || name.starts_with("Steamworks")
                || name.starts_with("Proton")
                || name.starts_with("Steam Linux Runtime")
            {
                continue;
            }
            let dir = apps.join("common").join(install);
            let exe = exe_candidates(&dir)
                .into_iter()
                .next()
                .map(|c| PathBuf::from(c.path));
            games.push(game(
                format!("steam:{app_id}"),
                name.to_owned(),
                GameSource::Steam {
                    app_id: app_id.to_owned(),
                },
                Some(dir),
                exe,
            ));
        }
    }
    games
}

pub fn scan_epic() -> Vec<Game> {
    let Some(program_data) = std::env::var_os("ProgramData") else {
        return Vec::new();
    };
    let manifests = PathBuf::from(program_data).join(r"Epic\EpicGamesLauncher\Data\Manifests");
    let Ok(dir) = std::fs::read_dir(manifests) else {
        return Vec::new();
    };

    dir.flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("item")))
        .filter_map(|e| serde_json::from_slice::<serde_json::Value>(&std::fs::read(e.path()).ok()?).ok())
        .filter(|m| !m["bIsIncompleteInstall"].as_bool().unwrap_or(false))
        .filter(|m| match m["AppCategories"].as_array() {
            Some(cats) => cats.iter().any(|c| c.as_str() == Some("games")),
            None => true,
        })
        .filter_map(|m| {
            let app_name = m["AppName"].as_str()?.to_owned();
            let name = m["DisplayName"].as_str().unwrap_or(&app_name).to_owned();
            let dir = PathBuf::from(m["InstallLocation"].as_str()?);
            let exe = m["LaunchExecutable"].as_str().filter(|s| !s.is_empty()).map(|s| dir.join(s));
            let launch_uri = match (m["CatalogNamespace"].as_str(), m["CatalogItemId"].as_str()) {
                (Some(ns), Some(item)) => format!("com.epicgames.launcher://apps/{ns}%3A{item}%3A{app_name}?action=launch&silent=true"),
                _ => format!("com.epicgames.launcher://apps/{app_name}?action=launch&silent=true"),
            };
            Some(game(format!("epic:{app_name}"), name, GameSource::Epic { app_name, launch_uri }, Some(dir), exe))
        })
        .collect()
}

const SKIP_DIRS: &[&str] = &[
    "_commonredist",
    "redist",
    "redistributables",
    "__installer",
    "directx",
    "support",
    "easyanticheat",
    "battleye",
    "prereqs",
];
const SKIP_EXE: &[&str] = &[
    "unins",
    "setup",
    "redist",
    "crash",
    "report",
    "dxsetup",
    "dotnet",
    "prereq",
    "easyanticheat",
    "battleye",
    "launcherpatcher",
    "cefprocess",
    "helper",
];

/// Largest executables first: the real game binary (e.g. *-Win64-Shipping.exe)
/// is almost always much bigger than bootstrappers and tools.
pub fn exe_candidates(dir: &Path) -> Vec<ExeCandidate> {
    fn walk(dir: &Path, depth: u8, out: &mut Vec<ExeCandidate>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                if depth > 0 && !SKIP_DIRS.contains(&name.as_str()) {
                    walk(&path, depth - 1, out);
                }
            } else if name.ends_with(".exe") && !SKIP_EXE.iter().any(|s| name.contains(s)) {
                out.push(ExeCandidate {
                    path: path.display().to_string(),
                    size: meta.len(),
                });
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, 5, &mut out);
    out.sort_by_key(|c| std::cmp::Reverse(c.size));
    out.truncate(30);
    out
}

pub fn manual(exe: &Path, name: Option<String>) -> Game {
    let name = name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| {
        exe.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Игра".into())
    });
    let id = format!("manual:{}", exe.display().to_string().to_lowercase());
    game(
        id,
        name,
        GameSource::Manual,
        exe.parent().map(Path::to_path_buf),
        Some(exe.to_path_buf()),
    )
}
