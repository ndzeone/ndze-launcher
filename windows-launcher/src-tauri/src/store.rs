use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::reg::RegData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Priority {
    #[default]
    Normal,
    AboveNormal,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoostPlan {
    #[default]
    High,
    Ultimate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameSource {
    Steam {
        app_id: String,
    },
    Epic {
        app_name: String,
        launch_uri: String,
    },
    Manual,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameProfile {
    pub high_perf_gpu: bool,
    pub disable_fso: bool,
    pub priority: Priority,
    pub boost_power_plan: bool,
    pub close_processes: Vec<String>,
    pub launch_args: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameStats {
    pub playtime_secs: u64,
    pub launches: u32,
    pub last_played: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub source: GameSource,
    pub install_dir: Option<String>,
    pub exe_path: Option<String>,
    #[serde(default)]
    pub profile: GameProfile,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub stats: GameStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub boost_plan: BoostPlan,
    pub ultimate_guid: Option<String>,
    pub minimize_on_launch: bool,
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub games: BTreeMap<String, Game>,
    pub backups: BTreeMap<String, Vec<Option<RegData>>>,
    pub settings: Settings,
}

pub struct Store {
    path: PathBuf,
    pub config: Config,
}

impl Store {
    pub fn load(path: PathBuf) -> Self {
        let config = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, config }
    }

    /// Write-then-rename so a crash mid-save never corrupts the registry backups.
    pub fn save(&self) -> AppResult<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(&self.config)?)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(())
    }
}
