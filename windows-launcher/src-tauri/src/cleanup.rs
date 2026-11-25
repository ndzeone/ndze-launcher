use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{fail, AppResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanTarget {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub size: u64,
    pub files: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub freed: u64,
    pub skipped: u64,
}

struct Target {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    dirs: fn() -> Vec<PathBuf>,
}

fn local(parts: &[&str]) -> Vec<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|base| parts.iter().map(|p| PathBuf::from(&base).join(p)).collect())
        .unwrap_or_default()
}

const TARGETS: &[Target] = &[
    Target {
        id: "temp",
        title: "Временные файлы",
        description: "Папка %TEMP%: остатки установщиков, распаковок и логов. Занятые файлы пропускаются.",
        dirs: || {
            let temp = std::env::temp_dir();
            let is_temp = temp.file_name().is_some_and(|n| n.eq_ignore_ascii_case("temp"));
            if is_temp { vec![temp] } else { Vec::new() }
        },
    },
    Target {
        id: "shader_nvidia",
        title: "Кэш шейдеров NVIDIA",
        description: "Чистите после обновления драйвера или при статтерах. Первые запуски игр после очистки будут чуть медленнее, пока кэш не соберётся заново.",
        dirs: || local(&[r"NVIDIA\DXCache", r"NVIDIA\GLCache"]),
    },
    Target {
        id: "shader_amd",
        title: "Кэш шейдеров AMD",
        description: "То же для видеокарт Radeon.",
        dirs: || local(&[r"AMD\DxCache", r"AMD\DxcCache", r"AMD\GLCache", r"AMD\VkCache"]),
    },
    Target {
        id: "shader_directx",
        title: "Кэш шейдеров DirectX",
        description: "Общий кэш Windows (D3DSCache). Пересоздаётся автоматически.",
        dirs: || local(&["D3DSCache"]),
    },
    Target {
        id: "crash_dumps",
        title: "Дампы падений",
        description: "Файлы .dmp от вылетевших программ — нужны только разработчикам.",
        dirs: || local(&["CrashDumps"]),
    },
];

fn measure(dir: &Path, size: &mut u64, files: &mut u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            measure(&entry.path(), size, files);
        } else {
            *size += meta.len();
            *files += 1;
        }
    }
}

fn wipe(dir: &Path, result: &mut CleanResult) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            wipe(&path, result);
            let _ = std::fs::remove_dir(&path);
        } else if std::fs::remove_file(&path).is_ok() {
            result.freed += meta.len();
        } else {
            result.skipped += 1;
        }
    }
}

pub fn scan() -> Vec<CleanTarget> {
    TARGETS
        .iter()
        .map(|t| {
            let (mut size, mut files) = (0, 0);
            for dir in (t.dirs)() {
                measure(&dir, &mut size, &mut files);
            }
            CleanTarget {
                id: t.id,
                title: t.title,
                description: t.description,
                size,
                files,
            }
        })
        .collect()
}

/// Never wipe anything shallower than e.g. C:\Users\name\AppData.
fn safe_root(dir: &Path) -> bool {
    dir.is_absolute() && dir.components().count() >= 4
}

pub fn run(ids: &[String]) -> AppResult<CleanResult> {
    let mut result = CleanResult {
        freed: 0,
        skipped: 0,
    };
    for id in ids {
        let Some(target) = TARGETS.iter().find(|t| t.id == id) else {
            return fail(format!("Неизвестная категория очистки: {id}"));
        };
        for dir in (target.dirs)().into_iter().filter(|d| safe_root(d)) {
            wipe(&dir, &mut result);
        }
    }
    Ok(result)
}
