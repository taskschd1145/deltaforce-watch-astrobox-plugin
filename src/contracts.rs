//! Shared product/data contract. All timestamps are Unix milliseconds; all money is integral.
use serde::{Deserialize, Serialize};

pub const PLUGIN_VERSION: &str = "0.1.0";
pub const STORE_VERSION: u32 = 1;
pub const DEMO_CAPTURED_AT: i64 = 1_794_717_600_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub name: String,
    pub level: u32,
    pub funds: u64,
    pub warehouse_capacity: u32,
    pub warehouse_used: u32,
    pub completed_runs: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    pub quality: String,
    pub category: String,
    pub quantity: u32,
    pub unit_value: u64,
    pub slots: u32,
    pub locked: bool,
    pub durability: Option<u32>,
    pub max_durability: Option<u32>,
    pub compartment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DamagePart {
    pub part: String,
    pub damage_taken: Option<u32>,
    pub injury_at_end: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatchRecord {
    pub id: String,
    /// operations or battlefield.
    pub mode: String,
    /// extracted/killed/timeout/victory/defeat/draw/practice/aborted.
    pub outcome: String,
    pub map_name: String,
    pub difficulty: String,
    pub started_at: Option<i64>,
    pub duration_seconds: u32,
    pub kills: u32,
    pub loot_value: u64,
    pub score: u32,
    pub xp_gain: u32,
    pub deaths: u32,
    pub captures: u32,
    pub revives: u32,
    pub loot: Vec<Item>,
    pub damage: Vec<DamagePart>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Facility {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub max_level: u32,
    pub description: String,
    pub effect: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub id: String,
    pub department: String,
    pub title: String,
    pub description: String,
    pub progress: u32,
    pub target: u32,
    pub reward: String,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    pub source: String,
    pub captured_at: i64,
    pub source_game_version: String,
    pub player: Player,
    pub matches: Vec<MatchRecord>,
    pub inventory: Vec<Item>,
    pub facilities: Vec<Facility>,
    pub missions: Vec<Mission>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettings {
    pub auto_backup_on_launch: bool,
    pub retention_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: String,
    pub created_at: i64,
    pub source: String,
    pub device_name: String,
    pub snapshot: Snapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedData {
    pub schema_version: u32,
    pub settings: PluginSettings,
    pub snapshot: Snapshot,
    pub backups: Vec<BackupRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Matches,
    MatchDetail,
    Services,
    Settings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirmation {
    Restore(String),
    DeleteBackup(String),
    SetRetention(u32),
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub data: PersistedData,
    pub page: Page,
    pub history_filter: String,
    pub selected_match_id: Option<String>,
    /// Preview is separate from the persisted active snapshot.
    pub viewed_backup_id: Option<String>,
    pub services_tab: String,
    pub inventory_query: String,
    pub inventory_quality: String,
    pub inventory_category: String,
    pub mission_filter: String,
    pub selected_item_id: Option<String>,
    pub connected: bool,
    pub auto_backup_pending: bool,
    pub auto_backup_done: bool,
    pub busy: bool,
    pub notice: String,
    pub retention_input: String,
    pub confirmation: Option<Confirmation>,
    pub timezone_offset_minutes: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    pub persist: bool,
    pub export_backup: Option<String>,
}
