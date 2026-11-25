use serde::Serialize;
use serde_json::Value;
use sysinfo::{CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub build: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub cores: usize,
    pub threads: usize,
    pub base_mhz: u64,
    pub l2_kb: Option<u64>,
    pub l3_kb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuDetail {
    pub name: String,
    pub vendor: String,
    pub vram: u64,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    pub slot: String,
    pub capacity: u64,
    pub speed: u32,
    pub configured_speed: u32,
    pub manufacturer: String,
    pub part: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total: u64,
    pub modules: Vec<MemoryModule>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub kind: String,
    pub file_system: String,
    pub total: u64,
    pub available: u64,
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub device: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh: u32,
    pub max_refresh: u32,
    pub rates: Vec<u32>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardInfo {
    pub vendor: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareReport {
    pub os: OsInfo,
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuDetail>,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub displays: Vec<DisplayInfo>,
    pub board: Option<BoardInfo>,
    pub laptop: bool,
    pub elevated: bool,
}

fn s(v: &Value) -> String {
    v.as_str().map(|x| x.trim().to_owned()).unwrap_or_default()
}

fn n(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|x| x.parse().ok()))
        .unwrap_or(0)
}

fn memory_kind(code: u64) -> &'static str {
    match code {
        20 => "DDR",
        21 => "DDR2",
        24 => "DDR3",
        26 => "DDR4",
        34 => "DDR5",
        _ => "",
    }
}

pub fn gpu_vendor(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("nvidia") || n.contains("geforce") || n.contains("rtx") || n.contains("gtx") {
        "NVIDIA"
    } else if n.contains("amd") || n.contains("radeon") {
        "AMD"
    } else if n.contains("intel") || n.contains("arc") {
        "Intel"
    } else {
        ""
    }
}

#[cfg(windows)]
fn platform() -> (Option<Value>, Vec<DisplayInfo>, Vec<crate::system::GpuInfo>) {
    (
        crate::win::cim_details(),
        crate::win::displays(),
        crate::win::gpus(),
    )
}

#[cfg(not(windows))]
fn platform() -> (Option<Value>, Vec<DisplayInfo>, Vec<crate::system::GpuInfo>) {
    (None, Vec::new(), Vec::new())
}

pub fn report() -> HardwareReport {
    let sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    );
    let (cim, displays, dxgi) = platform();
    let cim = cim.unwrap_or(Value::Null);

    let first_cpu = sys.cpus().first();
    let cpu = CpuInfo {
        name: first_cpu
            .map(|c| c.brand().trim().to_owned())
            .unwrap_or_default(),
        vendor: first_cpu
            .map(|c| c.vendor_id().to_owned())
            .unwrap_or_default(),
        cores: System::physical_core_count()
            .unwrap_or_else(|| n(&cim["cpu"]["NumberOfCores"]) as usize),
        threads: sys.cpus().len(),
        base_mhz: match n(&cim["cpu"]["MaxClockSpeed"]) {
            0 => first_cpu.map_or(0, |c| c.frequency()),
            mhz => mhz,
        },
        l2_kb: Some(n(&cim["cpu"]["L2CacheSize"])).filter(|&v| v > 0),
        l3_kb: Some(n(&cim["cpu"]["L3CacheSize"])).filter(|&v| v > 0),
    };

    let drivers: Vec<(String, String)> = cim["gpus"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|g| (s(&g["Name"]), s(&g["DriverVersion"])))
                .collect()
        })
        .unwrap_or_default();
    let gpus = dxgi
        .into_iter()
        .map(|g| GpuDetail {
            vendor: gpu_vendor(&g.name).to_owned(),
            driver: drivers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&g.name))
                .map(|(_, d)| d.clone())
                .filter(|d| !d.is_empty()),
            name: g.name,
            vram: g.vram,
        })
        .collect();

    let modules = cim["memory"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|m| MemoryModule {
                    slot: s(&m["DeviceLocator"]),
                    capacity: n(&m["Capacity"]),
                    speed: n(&m["Speed"]) as u32,
                    configured_speed: n(&m["ConfiguredClockSpeed"]) as u32,
                    manufacturer: s(&m["Manufacturer"]),
                    part: s(&m["PartNumber"]),
                    kind: memory_kind(n(&m["SMBIOSMemoryType"])).to_owned(),
                })
                .collect()
        })
        .unwrap_or_default();

    let disks = Disks::new_with_refreshed_list()
        .list()
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().into_owned(),
            mount: d.mount_point().display().to_string(),
            kind: match d.kind() {
                DiskKind::SSD => "ssd",
                DiskKind::HDD => "hdd",
                DiskKind::Unknown(_) => "unknown",
            }
            .to_owned(),
            file_system: d.file_system().to_string_lossy().into_owned(),
            total: d.total_space(),
            available: d.available_space(),
            removable: d.is_removable(),
        })
        .filter(|d| d.total > 0)
        .collect();

    let board = Some(BoardInfo {
        vendor: s(&cim["board"]["Manufacturer"]),
        name: s(&cim["board"]["Product"]),
    })
    .filter(|b| !b.name.is_empty());

    HardwareReport {
        os: OsInfo {
            name: Some(s(&cim["os"]["Caption"]))
                .map(|x| {
                    x.trim_start_matches("Майкрософт ")
                        .trim_start_matches("Microsoft ")
                        .to_owned()
                })
                .filter(|x| !x.is_empty())
                .or_else(System::long_os_version)
                .unwrap_or_default(),
            version: s(&cim["os"]["Version"]),
            build: s(&cim["os"]["BuildNumber"]),
            arch: s(&cim["os"]["OSArchitecture"]),
        },
        cpu,
        gpus,
        memory: MemoryInfo {
            total: sys.total_memory(),
            modules,
        },
        disks,
        displays,
        board,
        laptop: n(&cim["system"]["PCSystemType"]) == 2,
        elevated: crate::reg::is_elevated(),
    }
}
