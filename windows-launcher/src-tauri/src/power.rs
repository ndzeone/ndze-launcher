use serde::Serialize;

use crate::error::{fail, AppResult};

pub const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-cf2dd897d68f";
pub const POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
pub const ULTIMATE_TEMPLATE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerScheme {
    pub guid: String,
    pub name: String,
    pub active: bool,
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

fn label(guid: &str, raw: &str, ultimate: Option<&str>) -> String {
    match guid {
        BALANCED => "Сбалансированная".into(),
        HIGH_PERFORMANCE => "Высокая производительность".into(),
        POWER_SAVER => "Экономия энергии".into(),
        g if Some(g) == ultimate => "Максимальная производительность".into(),
        _ => raw.into(),
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
            Some(PowerScheme {
                name: label(&guid, raw, ultimate),
                active: line.trim_end().ends_with('*'),
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

/// Ultimate Performance is hidden on most editions; it has to be duplicated
/// from its template. The resulting GUID is cached so we never create copies twice.
pub fn ensure_ultimate(known: Option<&str>) -> AppResult<String> {
    if let Some(guid) = known {
        if list(Some(guid))?.iter().any(|s| s.guid == guid) {
            return Ok(guid.to_owned());
        }
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
}
