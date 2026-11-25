//! Heuristic "what to do with this PC" engine. Scores are rough tiers, not
//! benchmarks: they only need to be good enough to spot imbalances.

use serde::Serialize;

use crate::hardware::{gpu_vendor, HardwareReport};
use crate::store::Game;
use crate::tweaks::TweakStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Good,
    Tip,
    Warn,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub key: &'static str,
    pub label: &'static str,
    pub value: u8,
    pub caption: String,
}

#[derive(Debug, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AdviceAction {
    Tweaks { ids: Vec<String> },
    Refresh { device: String, hz: u32 },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdviceItem {
    pub id: String,
    pub tone: Tone,
    pub title: String,
    pub text: String,
    pub action: Option<AdviceAction>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSetting {
    pub label: &'static str,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    pub overall: u8,
    pub tier: &'static str,
    pub verdict_title: String,
    pub verdict_text: String,
    pub scores: Vec<Score>,
    pub settings: Vec<GameSetting>,
    pub items: Vec<AdviceItem>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GpuTraits {
    pub nvidia_rtx: bool,
    pub rtx_gen: u32,
    pub nvidia: bool,
    pub amd: bool,
    pub hags_capable: bool,
}

fn digits_after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let rest = &text[text.find(marker)? + marker.len()..];
    let rest = rest.trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

pub fn gpu_score(name: &str, vram: u64) -> (u8, GpuTraits) {
    let n = name.to_lowercase();
    let vram_gb = vram as f64 / 1024f64.powi(3);
    let suffix = if n.contains(" ti") { 4 } else { 0 }
        + if n.contains("super") || n.contains(" xt") {
            3
        } else {
            0
        };
    let mut traits = GpuTraits {
        nvidia: gpu_vendor(name) == "NVIDIA",
        amd: gpu_vendor(name) == "AMD",
        ..Default::default()
    };

    let score: i32 = if let Some(model) = digits_after(&n, "rtx").filter(|m| m.len() == 4) {
        let generation: u32 = model[..2].parse().unwrap_or(0);
        let tier: i32 = model[2..].parse().unwrap_or(60);
        traits.nvidia_rtx = true;
        traits.rtx_gen = generation;
        traits.hags_capable = true;
        let gen_bonus = match generation {
            50.. => 8,
            40 => 3,
            30 => -5,
            _ => -15,
        };
        tier + gen_bonus - 5 + suffix
    } else if let Some(model) = digits_after(&n, "gtx") {
        traits.hags_capable = model.len() == 4;
        let tier: i32 = model
            .get(model.len() - 2..)
            .and_then(|t| t.parse().ok())
            .unwrap_or(50);
        let gen_bonus = if model.starts_with("16") { -25 } else { -30 };
        tier + gen_bonus + suffix
    } else if let Some(model) = digits_after(&n, "rx").filter(|m| m.len() == 4) {
        let generation: u32 = model[..1].parse().unwrap_or(0);
        let tier: i32 = if generation == 9 {
            model[2..3].parse::<i32>().unwrap_or(6) * 10
        } else {
            model[1..2].parse::<i32>().unwrap_or(6) * 10
        };
        traits.hags_capable = generation >= 5;
        let gen_bonus = match generation {
            9 => 0,
            7 => -7,
            6 => -15,
            _ => -25,
        };
        tier + gen_bonus + suffix
    } else if n.contains("arc") {
        traits.hags_capable = true;
        let model = n.split_whitespace().find_map(|w| {
            w.strip_prefix('a')
                .or_else(|| w.strip_prefix('b'))
                .filter(|r| r.len() == 3 && r.chars().all(|c| c.is_ascii_digit()))
        });
        match model {
            Some(m) if m.starts_with('7') => 42,
            Some(m) if m.starts_with('5') => 36,
            _ => 25,
        }
    } else if vram_gb < 1.5 {
        10
    } else {
        (vram_gb * 5.0) as i32
    };
    (score.clamp(5, 100) as u8, traits)
}

pub fn cpu_score(name: &str, threads: usize) -> u8 {
    let n = name.to_lowercase();
    let base: i32 = match threads {
        0..=4 => 22,
        5..=8 => 40,
        9..=12 => 55,
        13..=16 => 66,
        17..=24 => 76,
        _ => 84,
    };
    let mut bonus = 0;
    if let Some(model) = digits_after(&n, "-").filter(|m| m.len() >= 4) {
        if n.contains("intel") || n.contains("core") {
            let generation: i32 = if model.len() == 5 {
                model[..2].parse().unwrap_or(10)
            } else {
                model[..1].parse().unwrap_or(7)
            };
            bonus += match generation {
                14.. => 12,
                12 | 13 => 9,
                10 | 11 => 2,
                8 | 9 => -4,
                _ => -12,
            };
        }
    }
    if n.contains("core(tm) ultra") || n.contains("core ultra") {
        bonus += 10;
    }
    if let Some(series) = digits_after(&n, "ryzen").and_then(|_| {
        n.split_whitespace()
            .find(|w| w.len() >= 4 && w.chars().take(4).all(|c| c.is_ascii_digit()))
    }) {
        bonus += match series.chars().next() {
            Some('9') => 12,
            Some('7') => 9,
            Some('5') => 3,
            Some('3') => -5,
            _ => -12,
        };
    }
    if n.contains("x3d") {
        bonus += 12;
    }
    (base + bonus).clamp(5, 100) as u8
}

fn ram_score(report: &HardwareReport) -> (u8, String) {
    let gb = (report.memory.total as f64 / 1024f64.powi(3)).round() as u64;
    let mut score: i32 = match gb {
        0..=7 => 15,
        8..=11 => 35,
        12..=15 => 50,
        16..=31 => 62,
        32..=63 => 86,
        _ => 95,
    };
    let modules = &report.memory.modules;
    if modules.len() == 1 {
        score -= 15;
    }
    let speed = modules
        .iter()
        .map(|m| m.configured_speed)
        .max()
        .unwrap_or(0);
    let kind = modules.first().map(|m| m.kind.as_str()).unwrap_or("");
    if xmp_off(kind, speed) {
        score -= 6;
    }
    let caption = match (kind, speed) {
        ("", 0) => format!("{gb} ГБ"),
        (k, 0) => format!("{gb} ГБ {k}"),
        (k, s) => format!("{gb} ГБ {k}-{s}"),
    };
    (score.clamp(5, 100) as u8, caption)
}

fn xmp_off(kind: &str, speed: u32) -> bool {
    match kind {
        "DDR4" => speed > 0 && speed <= 2400,
        "DDR5" => speed > 0 && speed <= 4800,
        _ => false,
    }
}

fn game_disks<'a>(
    report: &'a HardwareReport,
    games: &[Game],
) -> Vec<(&'a crate::hardware::DiskInfo, Vec<String>)> {
    let mut result: Vec<(&crate::hardware::DiskInfo, Vec<String>)> = Vec::new();
    for game in games {
        let Some(dir) = game.install_dir.as_deref() else {
            continue;
        };
        let disk = report
            .disks
            .iter()
            .filter(|d| dir.to_lowercase().starts_with(&d.mount.to_lowercase()))
            .max_by_key(|d| d.mount.len());
        if let Some(disk) = disk {
            match result.iter_mut().find(|(d, _)| d.mount == disk.mount) {
                Some((_, names)) => names.push(game.name.clone()),
                None => result.push((disk, vec![game.name.clone()])),
            }
        }
    }
    result
}

fn plural(n: usize, one: &str, few: &str, many: &str) -> String {
    let form = match (n % 10, n % 100) {
        (1, r) if r != 11 => one,
        (2..=4, r) if !(12..=14).contains(&r) => few,
        _ => many,
    };
    format!("{n} {form}")
}

fn tier_label(score: u8) -> &'static str {
    match score {
        85.. => "Топовый",
        70..=84 => "Мощный",
        55..=69 => "Средний+",
        40..=54 => "Средний",
        25..=39 => "Начальный",
        _ => "Офисный",
    }
}

pub fn analyze(report: &HardwareReport, games: &[Game], tweaks: &[TweakStatus]) -> Advice {
    let gpu = report.gpus.iter().max_by_key(|g| g.vram);
    let (gpu_value, traits) =
        gpu.map_or((10, GpuTraits::default()), |g| gpu_score(&g.name, g.vram));
    let cpu_value = cpu_score(&report.cpu.name, report.cpu.threads);
    let (ram_value, ram_caption) = ram_score(report);

    let disks = game_disks(report, games);
    let hdd_games: Vec<String> = disks
        .iter()
        .filter(|(d, _)| d.kind == "hdd")
        .flat_map(|(_, g)| g.clone())
        .collect();
    let storage_value: u8 = if !hdd_games.is_empty() {
        45
    } else if disks.iter().any(|(d, _)| d.kind == "ssd")
        || report.disks.iter().any(|d| d.kind == "ssd")
    {
        90
    } else {
        65
    };

    let gpu_name = gpu.map_or("—".to_owned(), |g| {
        g.name.replace("NVIDIA ", "").replace("AMD ", "")
    });
    let cpu_short = report
        .cpu
        .name
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("12th Gen ", "")
        .replace("13th Gen ", "")
        .replace("14th Gen ", "");
    let scores = vec![
        Score {
            key: "cpu",
            label: "Процессор",
            value: cpu_value,
            caption: format!(
                "{} · {}",
                plural(report.cpu.cores, "ядро", "ядра", "ядер"),
                plural(report.cpu.threads, "поток", "потока", "потоков")
            ),
        },
        Score {
            key: "gpu",
            label: "Видеокарта",
            value: gpu_value,
            caption: gpu_name.clone(),
        },
        Score {
            key: "ram",
            label: "Память",
            value: ram_value,
            caption: ram_caption,
        },
        Score {
            key: "storage",
            label: "Накопитель",
            value: storage_value,
            caption: if hdd_games.is_empty() {
                "Игры на SSD".into()
            } else {
                "Есть игры на HDD".into()
            },
        },
    ];

    let gaming = (gpu_value as u32 * 50
        + cpu_value as u32 * 30
        + ram_value as u32 * 12
        + storage_value as u32 * 8)
        / 100;
    let overall = gaming.min(gpu_value as u32 + 15).min(100) as u8;
    let gap = gpu_value as i32 - cpu_value as i32;

    let (verdict_title, verdict_text) = if gap >= 15 {
        (
            "Упор в процессор".to_owned(),
            format!(
                "{gpu_name} заметно мощнее, чем {}. В динамичных и онлайн-играх FPS будет ограничивать CPU, а видеокарта — недогружена. \
                 Поэтому высокие разрешение и графика почти «бесплатны»: ставьте их смело. Помогают Frame Generation, ограничение FPS и закрытие фоновых программ. \
                 Самый заметный апгрейд — процессор с 6+ ядрами на той же платформе.",
                cpu_short.trim()
            ),
        )
    } else if gap <= -15 {
        (
            "Упор в видеокарту".to_owned(),
            "Процессор сильнее видеокарты: FPS ограничивает GPU. Снижайте разрешение рендера через DLSS/FSR, тени и объёмные эффекты — это даст больше всего кадров.".to_owned(),
        )
    } else {
        (
            "Сбалансированная сборка".to_owned(),
            "Процессор и видеокарта подходят друг другу — ни один компонент не тормозит другой. Настройки ниже выжмут максимум из этой связки.".to_owned(),
        )
    };

    let display = report
        .displays
        .iter()
        .find(|d| d.primary)
        .or_else(|| report.displays.first());
    let settings = game_settings(
        display,
        gpu_value,
        cpu_value,
        gap,
        traits,
        gpu.map_or(0, |g| g.vram),
    );
    let items = advice_items(report, traits, &hdd_games, tweaks);

    Advice {
        overall,
        tier: tier_label(overall),
        verdict_title,
        verdict_text,
        scores,
        settings,
        items,
    }
}

fn game_settings(
    display: Option<&crate::hardware::DisplayInfo>,
    gpu: u8,
    cpu: u8,
    gap: i32,
    traits: GpuTraits,
    vram: u64,
) -> Vec<GameSetting> {
    let (width, height, hz) = display.map_or((1920, 1080, 60), |d| {
        (d.width, d.height, d.max_refresh.max(d.refresh))
    });
    let pixels = width as f64 * height as f64;
    let penalty = if pixels > 6.0e6 {
        28
    } else if pixels > 4.0e6 {
        18
    } else if pixels > 2.5e6 {
        12
    } else {
        0
    };
    let effective = gpu as i32 - penalty;
    let preset = match effective {
        70.. => "Ультра",
        55..=69 => "Высокие",
        40..=54 => "Средние",
        25..=39 => "Низкие–средние",
        _ => "Низкие",
    };
    let vram_gb = (vram as f64 / 1024f64.powi(3)).round() as u64;
    let textures = match vram_gb {
        12.. => "Ультра",
        8..=11 => "Высокие",
        6..=7 => "Средние",
        _ => "Низкие",
    };
    let upscaler = match (traits.nvidia_rtx, pixels > 2.5e6 || effective < 55) {
        (true, true) => "DLSS «Качество»",
        (true, false) => "DLAA (или DLSS «Качество»)",
        (false, true) => "FSR / XeSS «Качество»",
        (false, false) => "Нативное + TAA/FSR Native AA",
    };
    let frame_gen = if traits.nvidia_rtx && traits.rtx_gen >= 40 {
        "DLSS Frame Generation"
    } else {
        "FSR 3 Frame Generation"
    };
    let frame_gen_note = if gap >= 15 {
        "Особенно полезно при упоре в CPU: генерация кадров идёт на видеокарте. В киберспорте лучше выключить — добавляет задержку."
    } else {
        "В одиночных играх для плавности. В соревновательных — выключите: добавляет задержку ввода."
    };
    let latency = if traits.nvidia {
        "NVIDIA Reflex: Вкл. + Boost"
    } else if traits.amd {
        "AMD Anti-Lag 2: Вкл."
    } else {
        "Режим низкой задержки в игре"
    };
    let cap = hz.saturating_sub(3).max(30);

    let mut settings = vec![
        GameSetting { label: "Разрешение", value: format!("{width}×{height}"), note: "Нативное разрешение монитора — масштабирование ниже делайте через апскейлер, а не через разрешение.".into() },
        GameSetting { label: "Пресет графики", value: preset.into(), note: format!("Под ~{hz} Гц на вашем мониторе. Сначала снижайте тени, объёмный туман и отражения.") },
        GameSetting { label: "Текстуры", value: textures.into(), note: format!("Зависят только от видеопамяти ({vram_gb} ГБ), на FPS почти не влияют.") },
        GameSetting { label: "Апскейлер", value: upscaler.into(), note: "Если FPS не хватает — переключите на «Баланс».".into() },
        GameSetting { label: "Генерация кадров", value: frame_gen.into(), note: frame_gen_note.into() },
        GameSetting { label: "Задержка", value: latency.into(), note: "Снижает input lag, особенно при полной загрузке видеокарты.".into() },
        GameSetting { label: "Ограничение FPS", value: format!("{cap} FPS"), note: "С G-Sync/FreeSync держите FPS на 3 ниже частоты монитора, VSync в игре — выкл.".into() },
    ];
    if cpu < 45 {
        settings.push(GameSetting {
            label: "Нагрузка на CPU",
            value: "Дальность прорисовки и толпы — средние".into(),
            note: "Именно эти параметры (NPC, физика, дальность объектов) нагружают процессор."
                .into(),
        });
    }
    settings
}

fn advice_items(
    report: &HardwareReport,
    traits: GpuTraits,
    hdd_games: &[String],
    tweaks: &[TweakStatus],
) -> Vec<AdviceItem> {
    let mut items = Vec::new();
    let mut push =
        |id: &str, tone: Tone, title: String, text: String, action: Option<AdviceAction>| {
            items.push(AdviceItem {
                id: id.into(),
                tone,
                title,
                text,
                action,
            });
        };

    for display in &report.displays {
        if display.max_refresh > display.refresh + 5 {
            push(
                &format!("refresh-{}", display.device),
                Tone::Warn,
                format!("Монитор работает на {} Гц из {} Гц", display.refresh, display.max_refresh),
                format!("{} поддерживает {} Гц при текущем разрешении, но Windows использует {} Гц. Это самая частая причина «игра на 60 FPS ощущается дёргано».", display.name, display.max_refresh, display.refresh),
                Some(AdviceAction::Refresh { device: display.device.clone(), hz: display.max_refresh }),
            );
        } else if display.refresh > 60 {
            push(
                &format!("refresh-{}", display.device),
                Tone::Good,
                format!("{} — {} Гц", display.name, display.refresh),
                "Монитор работает на максимальной частоте.".into(),
                None,
            );
        }
    }

    let modules = &report.memory.modules;
    match modules.len() {
        1 => push(
            "ram-single",
            Tone::Warn,
            "Одноканальный режим памяти".into(),
            "Установлена одна планка RAM. В двухканальном режиме (две одинаковые планки) FPS в играх, упирающихся в CPU, выше на 15–30%.".into(),
            None,
        ),
        2.. => push("ram-dual", Tone::Good, "Двухканальная память".into(), format!("{} планки — пропускная способность памяти используется полностью.", modules.len()), None),
        _ => {}
    }
    if let Some(module) = modules
        .iter()
        .find(|m| xmp_off(&m.kind, m.configured_speed))
    {
        push(
            "ram-xmp",
            Tone::Tip,
            format!("Память работает на {} МГц", module.configured_speed),
            format!("Для {} это базовая частота: похоже, профиль XMP/EXPO выключен. Включите его в BIOS — это бесплатный прирост FPS.", module.kind),
            None,
        );
    }
    let gb = (report.memory.total as f64 / 1024f64.powi(3)).round() as u64;
    if (12..=16).contains(&gb) {
        push("ram-16", Tone::Tip, "16 ГБ — впритык для новых игр".into(), "Перед игрой закрывайте браузер и мессенджеры (настраивается в профиле игры). Для современных AAA комфортнее 32 ГБ.".into(), None);
    } else if gb < 12 {
        push(
            "ram-low",
            Tone::Warn,
            format!("Всего {gb} ГБ оперативной памяти"),
            "Современным играм нужно минимум 16 ГБ, иначе будут подгрузки и фризы.".into(),
            None,
        );
    }

    if !hdd_games.is_empty() {
        push(
            "hdd-games",
            Tone::Warn,
            "Игры установлены на HDD".into(),
            format!(
                "{} — перенос на SSD уберёт долгие загрузки и подгрузки текстур в открытом мире.",
                hdd_games.join(", ")
            ),
            None,
        );
    }
    if let Some(system) = report
        .disks
        .iter()
        .find(|d| d.mount.to_uppercase().starts_with("C:"))
    {
        let free_gb = system.available / 1024u64.pow(3);
        if free_gb < 20 {
            push("disk-low", Tone::Warn, format!("На диске C: свободно {free_gb} ГБ"), "Windows и драйверам нужно место под кэш шейдеров и обновления. Загляните в раздел «Очистка».".into(), None);
        }
    }

    if report.laptop {
        push("laptop", Tone::Tip, "Ноутбук".into(), "Играйте от сети и выберите режим «Производительность» в утилите производителя — без этого GPU и CPU работают в урезанном режиме.".into(), None);
    }

    let mut wanted = vec!["game_mode", "game_dvr_off", "windowed_optimizations"];
    if traits.hags_capable {
        wanted.push("hags");
    }
    if report.laptop {
        wanted.push("power_throttling_off");
    }
    let missing: Vec<String> = wanted
        .iter()
        .filter(|id| tweaks.iter().any(|t| t.id == **id && !t.applied))
        .map(|id| (*id).to_owned())
        .collect();
    if missing.is_empty() {
        push(
            "tweaks",
            Tone::Good,
            "Рекомендованные настройки Windows применены".into(),
            "Игровой режим, отключение Game DVR, оптимизации для оконных игр уже включены.".into(),
            None,
        );
    } else {
        let names: Vec<&str> = tweaks
            .iter()
            .filter(|t| missing.iter().any(|m| m == t.id))
            .map(|t| t.title)
            .collect();
        push(
            "tweaks",
            Tone::Tip,
            format!("Рекомендуется включить: {}", missing.len()),
            names.join(" · "),
            Some(AdviceAction::Tweaks { ids: missing }),
        );
    }

    if traits.nvidia {
        push("nvcp", Tone::Tip, "Панель управления NVIDIA".into(), "«Режим управления электропитанием» → «Предпочтителен режим максимальной производительности», «Режим низкой задержки» → «Вкл.» для игр без Reflex.".into(), None);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn scores_known_hardware() {
        let (rtx4070, t) = gpu_score("NVIDIA GeForce RTX 4070", 12 * GB);
        assert!(t.nvidia_rtx && t.rtx_gen == 40 && t.hags_capable);
        let (rtx3060, _) = gpu_score("NVIDIA GeForce RTX 3060", 12 * GB);
        let (gtx1060, _) = gpu_score("NVIDIA GeForce GTX 1060 6GB", 6 * GB);
        let (rx7800, _) = gpu_score("AMD Radeon RX 7800 XT", 16 * GB);
        let (igpu, _) = gpu_score("Intel(R) UHD Graphics 770", 128 * 1024 * 1024);
        assert!(
            rtx4070 > rtx3060 && rtx3060 > gtx1060 && gtx1060 > igpu,
            "{rtx4070} {rtx3060} {gtx1060} {igpu}"
        );
        assert!(
            (rx7800 as i32 - rtx4070 as i32).abs() < 10,
            "{rx7800} vs {rtx4070}"
        );

        let i3 = cpu_score("12th Gen Intel(R) Core(TM) i3-12100F", 8);
        let r7 = cpu_score("AMD Ryzen 7 7800X3D 8-Core Processor", 16);
        let i9 = cpu_score("13th Gen Intel(R) Core(TM) i9-13900K", 32);
        assert!(r7 > i3 && i9 > i3, "{i3} {r7} {i9}");
        assert!(
            rtx4070 as i32 - i3 as i32 >= 15,
            "i3 + 4070 must be CPU bound: {i3} vs {rtx4070}"
        );
    }

    #[test]
    fn russian_plurals() {
        assert_eq!(plural(4, "ядро", "ядра", "ядер"), "4 ядра");
        assert_eq!(plural(8, "поток", "потока", "потоков"), "8 потоков");
        assert_eq!(plural(21, "ядро", "ядра", "ядер"), "21 ядро");
        assert_eq!(plural(12, "ядро", "ядра", "ядер"), "12 ядер");
    }

    #[test]
    fn detects_xmp_off() {
        assert!(xmp_off("DDR4", 2133));
        assert!(!xmp_off("DDR4", 3200));
        assert!(xmp_off("DDR5", 4800));
        assert!(!xmp_off("", 2133));
    }
}
