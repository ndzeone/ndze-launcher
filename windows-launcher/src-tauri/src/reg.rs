use serde::{Deserialize, Serialize};

use crate::error::{fail, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hive {
    Hkcu,
    Hklm,
}

impl Hive {
    fn short(self) -> &'static str {
        match self {
            Hive::Hkcu => "HKCU",
            Hive::Hklm => "HKLM",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum RegData {
    Dword(u32),
    Sz(String),
}

impl RegData {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RegData::Sz(s) => Some(s),
            RegData::Dword(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegOp {
    pub hive: Hive,
    pub path: String,
    pub name: String,
    pub data: Option<RegData>,
}

pub use imp::{is_elevated, read};

pub fn read_str(hive: Hive, path: &str, name: &str) -> AppResult<Option<String>> {
    Ok(read(hive, path, name)?.and_then(|d| d.as_str().map(str::to_owned)))
}

/// HKLM changes go first through a single UAC prompt (unless already elevated),
/// so a cancelled prompt never leaves the HKCU half applied.
pub fn apply(ops: &[RegOp]) -> AppResult<()> {
    let (machine, user): (Vec<&RegOp>, Vec<&RegOp>) =
        ops.iter().partition(|op| op.hive == Hive::Hklm);

    if !machine.is_empty() {
        if is_elevated() {
            machine.iter().try_for_each(|op| apply_one(op))?;
        } else {
            imp::run_elevated_script(&elevated_script(&machine)?)?;
        }
    }
    user.iter().try_for_each(|op| apply_one(op))
}

fn apply_one(op: &RegOp) -> AppResult<()> {
    match &op.data {
        Some(data) => imp::write(op.hive, &op.path, &op.name, data),
        None => imp::delete(op.hive, &op.path, &op.name),
    }
}

/// Everything interpolated into the elevated script is restricted to a
/// conservative charset, so values from a tampered config cannot inject commands.
fn ensure_safe(value: &str) -> AppResult<()> {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '_' | '-' | '\\'))
        && !value.ends_with('\\')
    {
        Ok(())
    } else {
        fail(format!(
            "Недопустимые символы для системного реестра: {value}"
        ))
    }
}

pub fn elevated_script(ops: &[&RegOp]) -> AppResult<String> {
    let mut script = String::from("@echo off\r\n");
    for op in ops {
        let key = format!("{}\\{}", op.hive.short(), op.path);
        ensure_safe(&key)?;
        ensure_safe(&op.name)?;
        let name = &op.name;
        let line = match &op.data {
            Some(RegData::Dword(v)) => {
                format!("reg add \"{key}\" /v \"{name}\" /t REG_DWORD /d {v} /f >nul || exit /b 1")
            }
            Some(RegData::Sz(v)) => {
                ensure_safe(v)?;
                format!("reg add \"{key}\" /v \"{name}\" /t REG_SZ /d \"{v}\" /f >nul || exit /b 1")
            }
            None => format!("reg delete \"{key}\" /v \"{name}\" /f >nul 2>&1"),
        };
        script.push_str(&line);
        script.push_str("\r\n");
    }
    script.push_str("exit /b 0\r\n");
    Ok(script)
}

#[cfg(windows)]
mod imp {
    use std::io::ErrorKind;

    use winreg::enums::{RegType, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};
    use winreg::{RegKey, RegValue};

    use super::{Hive, RegData};
    use crate::error::{fail, AppResult};
    use crate::win;

    fn root(hive: Hive) -> RegKey {
        RegKey::predef(match hive {
            Hive::Hkcu => HKEY_CURRENT_USER,
            Hive::Hklm => HKEY_LOCAL_MACHINE,
        })
    }

    fn not_found(e: &std::io::Error) -> bool {
        e.kind() == ErrorKind::NotFound
    }

    pub fn read(hive: Hive, path: &str, name: &str) -> AppResult<Option<RegData>> {
        let key = match root(hive).open_subkey_with_flags(path, KEY_READ) {
            Ok(k) => k,
            Err(e) if not_found(&e) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        match key.get_raw_value(name) {
            Ok(raw) => Ok(decode(&raw)),
            Err(e) if not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn decode(value: &RegValue) -> Option<RegData> {
        let bytes: &[u8] = &value.bytes;
        match value.vtype {
            RegType::REG_DWORD => bytes
                .get(..4)
                .map(|b| RegData::Dword(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            RegType::REG_SZ | RegType::REG_EXPAND_SZ => {
                let wide: Vec<u16> = bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let text = String::from_utf16_lossy(&wide);
                Some(RegData::Sz(text.trim_end_matches('\0').to_owned()))
            }
            _ => None,
        }
    }

    pub fn write(hive: Hive, path: &str, name: &str, data: &RegData) -> AppResult<()> {
        let (key, _) = root(hive).create_subkey(path)?;
        match data {
            RegData::Dword(v) => key.set_value(name, v)?,
            RegData::Sz(s) => key.set_value(name, s)?,
        }
        Ok(())
    }

    pub fn delete(hive: Hive, path: &str, name: &str) -> AppResult<()> {
        let key = match root(hive).open_subkey_with_flags(path, KEY_SET_VALUE) {
            Ok(k) => k,
            Err(e) if not_found(&e) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        match key.delete_value(name) {
            Err(e) if !not_found(&e) => Err(e.into()),
            _ => Ok(()),
        }
    }

    pub fn is_elevated() -> bool {
        win::is_elevated()
    }

    pub fn run_elevated_script(script: &str) -> AppResult<()> {
        let path = std::env::temp_dir().join(format!("ndze-launcher-{}.cmd", std::process::id()));
        std::fs::write(&path, script)?;
        let params = format!("/d /c \"\"{}\"\"", path.display());
        let result = win::shell_execute("runas", "cmd.exe", Some(&params), None, false, true);
        let _ = std::fs::remove_file(&path);
        match result? {
            Some(0) => Ok(()),
            Some(code) => fail(format!("reg.exe завершился с кодом {code}")),
            None => fail("Не удалось дождаться завершения reg.exe"),
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::{Hive, RegData};
    use crate::error::{fail, AppResult};

    pub fn read(_: Hive, _: &str, _: &str) -> AppResult<Option<RegData>> {
        Ok(None)
    }
    pub fn write(_: Hive, _: &str, _: &str, _: &RegData) -> AppResult<()> {
        fail("Реестр доступен только в Windows")
    }
    pub fn delete(_: Hive, _: &str, _: &str) -> AppResult<()> {
        fail("Реестр доступен только в Windows")
    }
    pub fn is_elevated() -> bool {
        false
    }
    pub fn run_elevated_script(_: &str) -> AppResult<()> {
        fail("Повышение прав доступно только в Windows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(path: &str, name: &str, data: Option<RegData>) -> RegOp {
        RegOp {
            hive: Hive::Hklm,
            path: path.into(),
            name: name.into(),
            data,
        }
    }

    #[test]
    fn script_quotes_paths_and_values() {
        let a = op(
            r"SOFTWARE\Microsoft\Windows NT\X",
            "GPU Priority",
            Some(RegData::Dword(8)),
        );
        let b = op(
            r"SOFTWARE\Y",
            "Scheduling Category",
            Some(RegData::Sz("High".into())),
        );
        let c = op(r"SOFTWARE\Z", "Old", None);
        let s = elevated_script(&[&a, &b, &c]).unwrap();
        assert!(s.contains(r#"reg add "HKLM\SOFTWARE\Microsoft\Windows NT\X" /v "GPU Priority" /t REG_DWORD /d 8 /f"#));
        assert!(s.contains(r#"/t REG_SZ /d "High" /f"#));
        assert!(s.contains(r#"reg delete "HKLM\SOFTWARE\Z" /v "Old" /f"#));
    }

    #[test]
    fn script_rejects_injection() {
        let bad = op(
            r"SOFTWARE\Y",
            "V",
            Some(RegData::Sz("x\" & calc & \"".into())),
        );
        assert!(elevated_script(&[&bad]).is_err());
        let bad_path = op(r"SOFTWARE\Y%PATH%", "V", None);
        assert!(elevated_script(&[&bad_path]).is_err());
    }
}
