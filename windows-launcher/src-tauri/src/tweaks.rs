use serde::Serialize;

use crate::error::{fail, AppResult};
use crate::profile::{get_kv, set_kv, GPU_PREFS};
use crate::reg::{self, Hive, RegData, RegOp};
use crate::store::Config;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Restart {
    None,
    Relogin,
    Reboot,
}

enum Change {
    /// Target value and the Windows default used when no backup exists (None = value absent).
    Value(RegData, Option<RegData>),
    /// One `key=value;` pair inside a shared string value.
    Pair(&'static str, &'static str),
}

struct Entry {
    hive: Hive,
    path: &'static str,
    name: &'static str,
    change: Change,
}

struct Tweak {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    impact: Impact,
    restart: Restart,
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakStatus {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub impact: Impact,
    pub restart: Restart,
    pub requires_admin: bool,
    pub applied: bool,
}

fn dword(
    hive: Hive,
    path: &'static str,
    name: &'static str,
    target: u32,
    default: Option<u32>,
) -> Entry {
    Entry {
        hive,
        path,
        name,
        change: Change::Value(RegData::Dword(target), default.map(RegData::Dword)),
    }
}

fn sz(hive: Hive, path: &'static str, name: &'static str, target: &str, default: &str) -> Entry {
    Entry {
        hive,
        path,
        name,
        change: Change::Value(
            RegData::Sz(target.into()),
            Some(RegData::Sz(default.into())),
        ),
    }
}

const MMCSS: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
const MMCSS_GAMES: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";

fn catalog() -> Vec<Tweak> {
    use Hive::{Hkcu, Hklm};
    vec![
        Tweak {
            id: "game_mode",
            title: "Игровой режим Windows",
            description: "Windows отдаёт приоритет игре: приостанавливает установку обновлений и уведомления, снижает фоновую нагрузку. Стабилизирует frametime.",
            impact: Impact::Medium,
            restart: Restart::None,
            entries: vec![
                dword(Hkcu, r"Software\Microsoft\GameBar", "AutoGameModeEnabled", 1, None),
                dword(Hkcu, r"Software\Microsoft\GameBar", "AllowAutoGameMode", 1, None),
            ],
        },
        Tweak {
            id: "game_dvr_off",
            title: "Отключить фоновую запись Xbox Game DVR",
            description: "Фоновая запись геймплея постоянно кодирует видео и отнимает ресурсы GPU. Сама Xbox Game Bar продолжит работать.",
            impact: Impact::Medium,
            restart: Restart::None,
            entries: vec![
                dword(Hkcu, r"System\GameConfigStore", "GameDVR_Enabled", 0, Some(1)),
                dword(Hkcu, r"Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0, None),
            ],
        },
        Tweak {
            id: "windowed_optimizations",
            title: "Оптимизации для игр в окне",
            description: "Переводит DX10/11-игры в оконном и borderless режиме на flip-модель: меньше задержка, работают VRR и Auto HDR. Аналог переключателя в «Параметры → Дисплей → Графика».",
            impact: Impact::Medium,
            restart: Restart::None,
            entries: vec![Entry {
                hive: Hkcu,
                path: GPU_PREFS,
                name: "DirectXUserGlobalSettings",
                change: Change::Pair("SwapEffectUpgradeEnable", "1"),
            }],
        },
        Tweak {
            id: "hags",
            title: "Аппаратное планирование GPU (HAGS)",
            description: "GPU сам управляет своей очередью команд. Нужен для DLSS Frame Generation; обычно снижает задержку. Требует NVIDIA GTX 10xx+/AMD RX 5000+ и свежий драйвер.",
            impact: Impact::Medium,
            restart: Restart::Reboot,
            entries: vec![dword(Hklm, r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", 2, None)],
        },
        Tweak {
            id: "power_throttling_off",
            title: "Отключить Power Throttling",
            description: "Запрещает Windows переводить фоновые процессы на энергоэффективные ядра/частоты. Заметно на ноутбуках и CPU Intel 12-го поколения и новее; на десктопах эффект минимален.",
            impact: Impact::Low,
            restart: Restart::Reboot,
            entries: vec![dword(Hklm, r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff", 1, None)],
        },
        Tweak {
            id: "mmcss_games",
            title: "Приоритет MMCSS для игр",
            description: "Повышает приоритет задачи «Games» в планировщике мультимедиа и уменьшает резерв CPU для фона (20% → 10%). Эффект небольшой и зависит от игры.",
            impact: Impact::Low,
            restart: Restart::Reboot,
            entries: vec![
                dword(Hklm, MMCSS, "SystemResponsiveness", 10, Some(20)),
                dword(Hklm, MMCSS_GAMES, "GPU Priority", 8, Some(8)),
                dword(Hklm, MMCSS_GAMES, "Priority", 6, Some(2)),
                sz(Hklm, MMCSS_GAMES, "Scheduling Category", "High", "Medium"),
                sz(Hklm, MMCSS_GAMES, "SFIO Priority", "High", "Normal"),
            ],
        },
        Tweak {
            id: "mouse_accel_off",
            title: "Отключить акселерацию мыши",
            description: "Выключает «Повышенную точность установки указателя»: движение мыши становится линейным. На FPS не влияет, но важно для стабильного аима.",
            impact: Impact::Low,
            restart: Restart::Relogin,
            entries: vec![
                sz(Hkcu, r"Control Panel\Mouse", "MouseSpeed", "0", "1"),
                sz(Hkcu, r"Control Panel\Mouse", "MouseThreshold1", "0", "6"),
                sz(Hkcu, r"Control Panel\Mouse", "MouseThreshold2", "0", "10"),
            ],
        },
    ]
}

fn find(id: &str) -> AppResult<Tweak> {
    catalog()
        .into_iter()
        .find(|t| t.id == id)
        .map_or_else(|| fail(format!("Неизвестный твик: {id}")), Ok)
}

fn current(entry: &Entry) -> AppResult<Option<RegData>> {
    reg::read(entry.hive, entry.path, entry.name)
}

fn is_applied(entry: &Entry, value: Option<&RegData>) -> bool {
    match &entry.change {
        Change::Value(target, _) => value == Some(target),
        Change::Pair(key, target) => {
            value.and_then(RegData::as_str).and_then(|s| get_kv(s, key)) == Some(*target)
        }
    }
}

fn pair_op(entry: &Entry, current: Option<&RegData>, key: &str, value: Option<&str>) -> RegOp {
    let merged = set_kv(
        current.and_then(RegData::as_str).unwrap_or_default(),
        key,
        value,
    );
    op(entry, (!merged.is_empty()).then_some(RegData::Sz(merged)))
}

fn op(entry: &Entry, data: Option<RegData>) -> RegOp {
    RegOp {
        hive: entry.hive,
        path: entry.path.into(),
        name: entry.name.into(),
        data,
    }
}

pub fn list() -> AppResult<Vec<TweakStatus>> {
    catalog()
        .into_iter()
        .map(|t| {
            let applied = t
                .entries
                .iter()
                .map(|e| Ok(is_applied(e, current(e)?.as_ref())))
                .collect::<AppResult<Vec<bool>>>()?
                .into_iter()
                .all(|ok| ok);
            Ok(TweakStatus {
                id: t.id,
                title: t.title,
                description: t.description,
                impact: t.impact,
                restart: t.restart,
                requires_admin: t.entries.iter().any(|e| e.hive == Hive::Hklm),
                applied,
            })
        })
        .collect()
}

pub fn apply(id: &str, config: &mut Config) -> AppResult<()> {
    apply_many(&[id.to_owned()], config)
}

/// All registry writes of all requested tweaks go through one `reg::apply`,
/// so several HKLM tweaks still cost the user a single UAC prompt.
pub fn apply_many(ids: &[String], config: &mut Config) -> AppResult<()> {
    let mut ops = Vec::new();
    let mut backups = Vec::new();
    for id in ids {
        let tweak = find(id)?;
        let values = tweak
            .entries
            .iter()
            .map(current)
            .collect::<AppResult<Vec<_>>>()?;
        ops.extend(
            tweak
                .entries
                .iter()
                .zip(&values)
                .map(|(entry, value)| match &entry.change {
                    Change::Value(target, _) => op(entry, Some(target.clone())),
                    Change::Pair(key, target) => pair_op(entry, value.as_ref(), key, Some(target)),
                }),
        );
        backups.push((id.clone(), values));
    }

    reg::apply(&ops)?;
    for (id, values) in backups {
        config.backups.entry(id).or_insert(values);
    }
    Ok(())
}

pub fn revert(id: &str, config: &mut Config) -> AppResult<()> {
    let tweak = find(id)?;
    let backup = config
        .backups
        .get(id)
        .filter(|b| b.len() == tweak.entries.len());

    let mut ops = Vec::with_capacity(tweak.entries.len());
    for (i, entry) in tweak.entries.iter().enumerate() {
        let original = backup.map(|b| b[i].clone());
        ops.push(match &entry.change {
            Change::Value(_, default) => op(entry, original.unwrap_or_else(|| default.clone())),
            Change::Pair(key, _) => {
                let previous = original
                    .flatten()
                    .and_then(|d| d.as_str().and_then(|s| get_kv(s, key)).map(str::to_owned));
                pair_op(entry, current(entry)?.as_ref(), key, previous.as_deref())
            }
        });
    }

    reg::apply(&ops)?;
    config.backups.remove(id);
    Ok(())
}
