//! Startup apps, toggled exactly like Task Manager does it: the Run entry stays,
//! and a StartupApproved flag (first byte 02 = on, 03 = off) is written next to it.

use serde::Serialize;

use crate::error::{fail, AppResult};
use crate::reg::{self, Hive, RegData, RegOp};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub machine: bool,
    pub enabled: bool,
}

fn hive(machine: bool) -> Hive {
    if machine {
        Hive::Hklm
    } else {
        Hive::Hkcu
    }
}

fn approved(machine: bool, name: &str) -> AppResult<bool> {
    Ok(match reg::read(hive(machine), APPROVED, name)? {
        Some(RegData::Binary(bytes)) => bytes.first().is_none_or(|b| b % 2 == 0),
        _ => true,
    })
}

pub fn list() -> AppResult<Vec<StartupItem>> {
    let mut items = Vec::new();
    for machine in [false, true] {
        for (name, data) in reg::list_values(hive(machine), RUN)? {
            let Some(command) = data.as_str().map(str::to_owned) else {
                continue;
            };
            items.push(StartupItem {
                enabled: approved(machine, &name)?,
                name,
                command,
                machine,
            });
        }
    }
    items.sort_by_key(|i| (!i.enabled, i.name.to_lowercase()));
    Ok(items)
}

pub fn set(name: &str, machine: bool, enabled: bool) -> AppResult<()> {
    if !reg::list_values(hive(machine), RUN)?
        .iter()
        .any(|(n, _)| n == name)
    {
        return fail(format!("Программа «{name}» не найдена в автозагрузке"));
    }
    let mut flag = vec![0u8; 12];
    flag[0] = if enabled { 0x02 } else { 0x03 };
    reg::apply(&[RegOp {
        hive: hive(machine),
        path: APPROVED.into(),
        name: name.into(),
        data: Some(RegData::Binary(flag)),
    }])
}
