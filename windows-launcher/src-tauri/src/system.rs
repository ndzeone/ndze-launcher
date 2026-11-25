use serde::Serialize;
use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vram: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStats {
    pub cpu: f32,
    pub cores: Vec<f32>,
    pub memory_used: u64,
    pub memory_total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub name: String,
    pub memory: u64,
    pub count: usize,
}

pub fn stats_system() -> System {
    System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    )
}

pub fn live(sys: &mut System) -> LiveStats {
    sys.refresh_cpu_usage();
    sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
    LiveStats {
        cpu: sys.global_cpu_usage(),
        cores: sys.cpus().iter().map(|c| c.cpu_usage()).collect(),
        memory_used: sys.used_memory(),
        memory_total: sys.total_memory(),
    }
}

pub fn processes() -> Vec<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    let mut grouped: std::collections::BTreeMap<String, ProcessInfo> = Default::default();
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy().into_owned();
        let entry = grouped.entry(name.to_lowercase()).or_insert(ProcessInfo {
            name,
            memory: 0,
            count: 0,
        });
        entry.memory += process.memory();
        entry.count += 1;
    }
    let mut list: Vec<_> = grouped.into_values().collect();
    list.sort_by_key(|p| std::cmp::Reverse(p.memory));
    list
}
