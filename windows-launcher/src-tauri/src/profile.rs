//! Per-executable settings stored by Windows itself (the same keys the
//! Settings app and the exe "Compatibility" tab write).

use crate::error::AppResult;
use crate::reg::{self, Hive, RegData, RegOp};

pub const GPU_PREFS: &str = r"Software\Microsoft\DirectX\UserGpuPreferences";
const LAYERS: &str = r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
const FSO_FLAG: &str = "DISABLEDXMAXIMIZEDWINDOWEDMODE";

pub fn get_kv<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    src.split(';')
        .filter_map(|pair| pair.split_once('='))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case(key))
        .map(|(_, v)| v.trim())
}

/// Updates one `key=value;` pair while preserving the others Windows stores there.
pub fn set_kv(src: &str, key: &str, value: Option<&str>) -> String {
    let mut pairs: Vec<(String, String)> = src
        .split(';')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .filter(|(k, _)| !k.is_empty() && !k.eq_ignore_ascii_case(key))
        .collect();
    if let Some(v) = value {
        pairs.push((key.to_owned(), v.to_owned()));
    }
    pairs.iter().map(|(k, v)| format!("{k}={v};")).collect()
}

pub fn set_layer_flag(src: &str, flag: &str, enabled: bool) -> String {
    let mut flags: Vec<&str> = src
        .split_whitespace()
        .filter(|f| *f != "~" && !f.eq_ignore_ascii_case(flag))
        .collect();
    if enabled {
        flags.push(flag);
    }
    if flags.is_empty() {
        String::new()
    } else {
        format!("~ {}", flags.join(" "))
    }
}

fn sz_op(path: &str, name: &str, value: String) -> RegOp {
    RegOp {
        hive: Hive::Hkcu,
        path: path.into(),
        name: name.into(),
        data: (!value.is_empty()).then_some(RegData::Sz(value)),
    }
}

pub fn apply(exe: &str, high_perf_gpu: bool, disable_fso: bool) -> AppResult<()> {
    let mut ops = Vec::new();

    let gpu = reg::read_str(Hive::Hkcu, GPU_PREFS, exe)?.unwrap_or_default();
    let ours = get_kv(&gpu, "GpuPreference") == Some("2");
    if high_perf_gpu && !ours {
        ops.push(sz_op(
            GPU_PREFS,
            exe,
            set_kv(&gpu, "GpuPreference", Some("2")),
        ));
    } else if !high_perf_gpu && ours {
        ops.push(sz_op(GPU_PREFS, exe, set_kv(&gpu, "GpuPreference", None)));
    }

    let layers = reg::read_str(Hive::Hkcu, LAYERS, exe)?.unwrap_or_default();
    let has_flag = layers
        .split_whitespace()
        .any(|f| f.eq_ignore_ascii_case(FSO_FLAG));
    if has_flag != disable_fso {
        ops.push(sz_op(
            LAYERS,
            exe,
            set_layer_flag(&layers, FSO_FLAG, disable_fso),
        ));
    }

    reg::apply(&ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_merge_keeps_other_pairs() {
        let src = "SwapEffectUpgradeEnable=1;GpuPreference=1;";
        assert_eq!(
            set_kv(src, "GpuPreference", Some("2")),
            "SwapEffectUpgradeEnable=1;GpuPreference=2;"
        );
        assert_eq!(
            set_kv(src, "GpuPreference", None),
            "SwapEffectUpgradeEnable=1;"
        );
        assert_eq!(get_kv(src, "gpupreference"), Some("1"));
        assert_eq!(set_kv("", "A", None), "");
    }

    #[test]
    fn layer_flags_merge() {
        assert_eq!(
            set_layer_flag("~ RUNASADMIN", FSO_FLAG, true),
            format!("~ RUNASADMIN {FSO_FLAG}")
        );
        assert_eq!(
            set_layer_flag(&format!("~ {FSO_FLAG}"), FSO_FLAG, false),
            ""
        );
        assert_eq!(set_layer_flag("", FSO_FLAG, true), format!("~ {FSO_FLAG}"));
    }
}
