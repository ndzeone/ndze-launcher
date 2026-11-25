use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::{fail, AppResult};

pub const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-cf2dd897d68f";
pub const POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
pub const ULTIMATE_TEMPLATE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

const BUILTIN: &[&str] = &[BALANCED, HIGH_PERFORMANCE, POWER_SAVER];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemeKind {
    Balanced,
    High,
    Saver,
    Ultimate,
    Custom,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerScheme {
    pub guid: String,
    pub name: String,
    pub kind: SchemeKind,
    pub active: bool,
    pub builtin: bool,
}

#[cfg(windows)]
fn powercfg(args: &[&str]) -> AppResult<String> {
    let output = crate::win::hidden_command("powercfg").args(args).output()?;
    let stdout = crate::win::decode_oem(&output.stdout);
    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = crate::win::decode_oem(&output.stderr);
        fail(format!(
            "powercfg {}: {}",
            args.join(" "),
            format!("{stdout}{stderr}").trim()
        ))
    }
}

#[cfg(not(windows))]
fn powercfg(_: &[&str]) -> AppResult<String> {
    fail("Схемы питания доступны только в Windows")
}

fn is_guid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// powercfg output is localized, so only GUIDs are parsed out of it.
pub fn extract_guids(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    (0..bytes.len().saturating_sub(35))
        .filter(|&i| text.is_char_boundary(i) && text.is_char_boundary(i + 36))
        .filter(|&i| i == 0 || !bytes[i - 1].is_ascii_hexdigit())
        .filter_map(|i| {
            let candidate = &text[i..i + 36];
            is_guid(candidate).then(|| candidate.to_ascii_lowercase())
        })
        .collect()
}

pub fn active() -> AppResult<String> {
    extract_guids(&powercfg(&["/getactivescheme"])?)
        .into_iter()
        .next()
        .map_or_else(|| fail("Не удалось определить активную схему питания"), Ok)
}

/// Copies of built-in schemes keep their template's name (in the OS language),
/// so both GUID and name are used to classify them.
fn classify(guid: &str, raw: &str, ultimate: Option<&str>) -> SchemeKind {
    let name = raw.to_lowercase();
    match guid {
        BALANCED => SchemeKind::Balanced,
        HIGH_PERFORMANCE => SchemeKind::High,
        POWER_SAVER => SchemeKind::Saver,
        g if Some(g) == ultimate => SchemeKind::Ultimate,
        _ if name.contains("ultimate") || name.contains("максимальная") => {
            SchemeKind::Ultimate
        }
        _ if name.contains("high performance") || name.contains("высокая производительность") => {
            SchemeKind::High
        }
        _ if name == "balanced" || name == "сбалансированная" => {
            SchemeKind::Balanced
        }
        _ if name.contains("power saver") || name.contains("экономия энергии") => {
            SchemeKind::Saver
        }
        _ => SchemeKind::Custom,
    }
}

fn label(kind: SchemeKind, raw: &str) -> String {
    match kind {
        SchemeKind::Balanced => "Сбалансированная".into(),
        SchemeKind::High => "Высокая производительность".into(),
        SchemeKind::Saver => "Экономия энергии".into(),
        SchemeKind::Ultimate => "Максимальная производительность".into(),
        SchemeKind::Custom => raw.into(),
    }
}

pub fn list(ultimate: Option<&str>) -> AppResult<Vec<PowerScheme>> {
    let out = powercfg(&["/list"])?;
    Ok(out
        .lines()
        .filter_map(|line| {
            let guid = extract_guids(line).into_iter().next()?;
            let raw = match (line.rfind('('), line.rfind(')')) {
                (Some(a), Some(b)) if a < b => line[a + 1..b].trim(),
                _ => guid.as_str(),
            };
            let kind = classify(&guid, raw, ultimate);
            Some(PowerScheme {
                name: label(kind, raw),
                kind,
                active: line.trim_end().ends_with('*'),
                builtin: BUILTIN.contains(&guid.as_str()),
                guid,
            })
        })
        .collect())
}

pub fn set_active(guid: &str) -> AppResult<()> {
    if !is_guid(guid) {
        return fail(format!("Некорректный GUID схемы питания: {guid}"));
    }
    powercfg(&["/setactive", guid]).map(|_| ())
}

pub fn delete(guid: &str, ultimate: Option<&str>) -> AppResult<()> {
    let scheme = list(ultimate)?
        .into_iter()
        .find(|s| s.guid == guid)
        .map_or_else(|| fail("Схема питания не найдена"), Ok)?;
    if scheme.builtin {
        return fail("Встроенные схемы Windows удалять нельзя");
    }
    if scheme.active {
        return fail("Нельзя удалить активную схему — сначала выберите другую");
    }
    powercfg(&["/delete", guid]).map(|_| ())
}

/// Keeps one scheme per kind (built-in, active or the remembered Ultimate first)
/// and deletes the extra copies. Custom schemes are never touched.
pub fn remove_duplicates(ultimate: Option<&str>) -> AppResult<usize> {
    let mut groups: BTreeMap<String, Vec<PowerScheme>> = BTreeMap::new();
    for scheme in list(ultimate)? {
        if scheme.kind != SchemeKind::Custom {
            groups.entry(scheme.name.clone()).or_default().push(scheme);
        }
    }
    let mut removed = 0;
    for mut group in groups.into_values().filter(|g| g.len() > 1) {
        group.sort_by_key(|s| (!s.builtin, !s.active, Some(s.guid.as_str()) != ultimate));
        for scheme in group
            .into_iter()
            .skip(1)
            .filter(|s| !s.builtin && !s.active)
        {
            powercfg(&["/delete", &scheme.guid])?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Ultimate Performance is hidden on most editions; it has to be duplicated
/// from its template. Existing copies are reused so we never create extra ones.
pub fn ensure_ultimate(known: Option<&str>) -> AppResult<String> {
    if let Some(existing) = list(known)?
        .into_iter()
        .find(|s| s.kind == SchemeKind::Ultimate)
    {
        return Ok(existing.guid);
    }
    let out = powercfg(&["-duplicatescheme", ULTIMATE_TEMPLATE])?;
    extract_guids(&out)
        .into_iter()
        .find(|g| g != ULTIMATE_TEMPLATE)
        .map_or_else(|| fail("Windows не вернул GUID новой схемы"), Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_guid_from_localized_output() {
        let out = "GUID схемы питания: 381b4222-f694-41f0-9685-ff5bb260df2e  (Сбалансированная) *";
        assert_eq!(extract_guids(out), vec![BALANCED.to_owned()]);
        assert!(extract_guids("nothing here").is_empty());
    }

    #[test]
    fn classifies_copies_by_name() {
        let g = "11111111-2222-3333-4444-555555555555";
        assert_eq!(
            classify(g, "Ultimate Performance", None),
            SchemeKind::Ultimate
        );
        assert_eq!(
            classify(g, "Максимальная производительность", None),
            SchemeKind::Ultimate
        );
        assert_eq!(classify(g, "High performance", None), SchemeKind::High);
        assert_eq!(classify(g, "My plan", None), SchemeKind::Custom);
        assert_eq!(classify(BALANCED, "whatever", None), SchemeKind::Balanced);
    }
}
