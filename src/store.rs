//! Local demo persistence. All writes are confined to the selected root/data directory.
use crate::contracts::*;
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const STATE_FILE: &str = "state.json";
pub const PREVIOUS_FILE: &str = "state.previous.json";
pub const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
const MAX_TIMESTAMP: i64 = 253_402_300_799_999;

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn text(value: &str, max: usize, field: &str) -> Result<(), String> {
    require(
        !value.trim().is_empty()
            && value.chars().count() <= max
            && !value.chars().any(char::is_control),
        &format!("{field} 必须为 1..{max} 个有效字符"),
    )
}
fn identifier(value: &str) -> Result<(), String> {
    require(
        !value.is_empty()
            && value.len() <= 96
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
        "id 必须为 1..96 个字母、数字、连字符、下划线或点",
    )
}
fn timestamp(value: i64, field: &str) -> Result<(), String> {
    require(
        (0..=MAX_TIMESTAMP).contains(&value),
        &format!("{field} 超出 Unix 毫秒时间范围"),
    )
}
fn unique<'a>(values: impl Iterator<Item = &'a str>, field: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        identifier(value)?;
        require(seen.insert(value), &format!("{field} 含重复 id：{value}"))?;
    }
    Ok(())
}

/// Validates integral item values, stack bounds, durability pairs and display enums.
pub fn validate_item(item: &Item) -> Result<(), String> {
    identifier(&item.id)?;
    text(&item.name, 120, "物资名称")?;
    require(
        ["白", "绿", "蓝", "紫", "金", "红"].contains(&item.quality.as_str()),
        "物资品质无效",
    )?;
    require(
        ["装备", "弹药", "药品", "贵重品", "房卡"].contains(&item.category.as_str()),
        "物资类别无效",
    )?;
    require(
        (1..=1_000_000).contains(&item.quantity) && (1..=10_000).contains(&item.slots),
        "物资数量或占格超出边界",
    )?;
    item.unit_value
        .checked_mul(u64::from(item.quantity))
        .ok_or_else(|| "物资总价溢出".to_string())?;
    match (item.durability, item.max_durability) {
        (None, None) => {}
        (Some(current), Some(maximum))
            if maximum > 0 && maximum <= 1_000_000 && current <= maximum => {}
        _ => return Err("物资耐久必须成对出现，且满足 0 ≤ 当前耐久 ≤ 最大耐久".into()),
    }
    require(
        ["warehouse", "safe", "backpack"].contains(&item.compartment.as_str()),
        "物资存放位置无效",
    )
}

/// Validates the complete mock snapshot without conflating score/XP or damage/injury.
pub fn validate_snapshot(snapshot: &Snapshot) -> Result<(), String> {
    require(
        snapshot.schema_version == STORE_VERSION,
        "快照 schemaVersion 不受支持",
    )?;
    require(snapshot.source == "mock", "UI Demo 快照 source 必须为 mock")?;
    timestamp(snapshot.captured_at, "capturedAt")?;
    text(&snapshot.source_game_version, 64, "sourceGameVersion")?;
    let player = &snapshot.player;
    text(&player.name, 80, "玩家名称")?;
    require((1..=1_000).contains(&player.level), "玩家等级超出边界")?;
    require(
        (1..=1_000_000).contains(&player.warehouse_capacity)
            && player.warehouse_used <= player.warehouse_capacity,
        "仓库容量或使用量无效",
    )?;
    require(player.completed_runs <= 1_000_000_000, "累计局数超出边界")?;
    require(
        snapshot.matches.len() <= 1_000
            && snapshot.inventory.len() <= 4_096
            && snapshot.facilities.len() <= 2
            && snapshot.missions.len() <= 256,
        "快照列表超出数量限制",
    )?;
    unique(snapshot.inventory.iter().map(|v| v.id.as_str()), "仓库")?;
    let mut used = 0u32;
    let mut inventory_value = 0u64;
    for item in &snapshot.inventory {
        validate_item(item)?;
        require(
            item.compartment == "warehouse",
            "仓库物资位置必须为 warehouse",
        )?;
        used = used
            .checked_add(item.slots)
            .ok_or_else(|| "仓库占格溢出".to_string())?;
        inventory_value = inventory_value
            .checked_add(
                item.unit_value
                    .checked_mul(u64::from(item.quantity))
                    .ok_or_else(|| "仓库物资总价溢出".to_string())?,
            )
            .ok_or_else(|| "仓库总价溢出".to_string())?;
    }
    require(
        used == player.warehouse_used,
        "仓库使用量必须等于物资占格总和",
    )?;
    unique(snapshot.matches.iter().map(|v| v.id.as_str()), "对局")?;
    for record in &snapshot.matches {
        require(record.source == "mock", "UI Demo 对局 source 必须为 mock")?;
        let allowed = match record.mode.as_str() {
            "operations" => {
                ["extracted", "killed", "timeout", "aborted"].contains(&record.outcome.as_str())
            }
            "battlefield" => ["victory", "defeat", "draw", "practice", "aborted"]
                .contains(&record.outcome.as_str()),
            _ => false,
        };
        require(allowed, "对局模式与结果组合无效")?;
        text(&record.map_name, 80, "地图")?;
        text(&record.difficulty, 40, "难度")?;
        if let Some(started) = record.started_at {
            timestamp(started, "startedAt")?;
            require(started <= snapshot.captured_at, "对局开始时间晚于快照时间")?;
        }
        require(
            record.duration_seconds <= 86_400
                && record.kills <= 1_000_000
                && record.deaths <= 1_000_000
                && record.captures <= 1_000_000
                && record.revives <= 1_000_000
                && record.score <= 1_000_000_000
                && record.xp_gain <= 1_000_000_000,
            "对局统计超出边界",
        )?;
        require(record.loot.len() <= 512, "对局战利品数量超出限制")?;
        unique(record.loot.iter().map(|v| v.id.as_str()), "战利品")?;
        let mut value = 0u64;
        for item in &record.loot {
            validate_item(item)?;
            require(
                item.compartment != "warehouse",
                "战利品位置必须为 safe 或 backpack",
            )?;
            if record.mode == "operations"
                && ["killed", "timeout", "aborted"].contains(&record.outcome.as_str())
            {
                require(item.compartment == "safe", "失败对局仅保留安全箱物资")?;
            }
            value = value
                .checked_add(
                    item.unit_value
                        .checked_mul(u64::from(item.quantity))
                        .ok_or_else(|| "战利品价值溢出".to_string())?,
                )
                .ok_or_else(|| "战利品总价溢出".to_string())?;
        }
        require(
            value == record.loot_value,
            "lootValue 必须等于 unitValue × quantity 的整数总和",
        )?;
        if record.mode == "battlefield" {
            require(
                record.loot.is_empty() && record.loot_value == 0,
                "全面战场战绩使用 score，战利品列表应为空",
            )?;
        }
        require(record.damage.len() == 5, "伤害分布必须包含五个部位")?;
        let mut parts = HashSet::new();
        for damage in &record.damage {
            require(
                ["head", "chest", "abdomen", "arm", "leg"].contains(&damage.part.as_str())
                    && parts.insert(damage.part.as_str()),
                "伤害部位无效或重复",
            )?;
            require(
                damage.damage_taken.map_or(true, |v| v <= 1_000_000)
                    && damage.injury_at_end.map_or(true, |v| v <= 100),
                "累计伤害或结束伤势超出边界",
            )?;
        }
    }
    unique(snapshot.facilities.iter().map(|v| v.id.as_str()), "设施")?;
    for facility in &snapshot.facilities {
        require(
            matches!(
                (facility.id.as_str(), facility.name.as_str()),
                ("warehouse", "仓库") | ("training-center", "训练中心")
            ),
            "演示设施限于仓库与训练中心",
        )?;
        require(
            facility.max_level > 0
                && facility.max_level <= 100
                && facility.level <= facility.max_level,
            "设施等级无效",
        )?;
        text(&facility.description, 1_024, "设施描述")?;
        text(&facility.effect, 1_024, "设施效果")?;
    }
    unique(snapshot.missions.iter().map(|v| v.id.as_str()), "任务")?;
    for mission in &snapshot.missions {
        require(
            mission.source == "demo-extension",
            "部门任务 source 必须为 demo-extension",
        )?;
        text(&mission.department, 80, "任务部门")?;
        text(&mission.title, 120, "任务名称")?;
        text(&mission.description, 2_048, "任务描述")?;
        text(&mission.reward, 240, "任务奖励")?;
        require(
            mission.target > 0 && mission.target <= 1_000_000 && mission.progress <= mission.target,
            "任务进度或目标无效",
        )?;
    }
    Ok(())
}

pub fn validate_backup(record: &BackupRecord) -> Result<(), String> {
    identifier(&record.id)?;
    timestamp(record.created_at, "createdAt")?;
    require(
        record.source == "mock" && record.source == record.snapshot.source,
        "备份来源与快照不匹配",
    )?;
    text(&record.device_name, 160, "deviceName")?;
    require(
        record.created_at >= record.snapshot.captured_at,
        "备份时间早于快照时间",
    )?;
    validate_snapshot(&record.snapshot)
}

pub fn validate_data(data: &PersistedData) -> Result<(), String> {
    require(
        data.schema_version == STORE_VERSION,
        "存储 schemaVersion 不受支持",
    )?;
    require(
        (1..=50).contains(&data.settings.retention_count),
        "备份保留数量必须为 1..50",
    )?;
    require(
        data.backups.len() <= data.settings.retention_count as usize,
        "备份数量超过当前保留策略",
    )?;
    validate_snapshot(&data.snapshot)?;
    unique(data.backups.iter().map(|v| v.id.as_str()), "备份")?;
    for record in &data.backups {
        validate_backup(record)?;
    }
    Ok(())
}

// A duplicate-key rejecting parser prevents ambiguous integrity/schema interpretation.
struct UniqueJson(Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<UniqueJson, E> {
                Ok(UniqueJson(Value::Bool(value)))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<UniqueJson, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<UniqueJson, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<UniqueJson, E> {
                serde_json::Number::from_f64(value)
                    .map(|n| UniqueJson(Value::Number(n)))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<UniqueJson, E> {
                Ok(UniqueJson(Value::String(value.into())))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<UniqueJson, E> {
                Ok(UniqueJson(Value::String(value)))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<UniqueJson, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<UniqueJson, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<UniqueJson, A::Error> {
                let mut result = Vec::new();
                while let Some(value) = seq.next_element::<UniqueJson>()? {
                    result.push(value.0);
                }
                Ok(UniqueJson(Value::Array(result)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<UniqueJson, A::Error> {
                let mut result = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if result.contains_key(&key) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON key: {key}"
                        )));
                    }
                    result.insert(key, map.next_value::<UniqueJson>()?.0);
                }
                Ok(UniqueJson(Value::Object(result)))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

/// Strict decode: duplicate, extra and missing keys are rejected recursively.
/// Comparing the typed serialization shape also covers nested array elements.
pub fn decode_strict<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T, String> {
    require(
        bytes.len() <= MAX_DOCUMENT_BYTES,
        "JSON 文档超过 16 MiB 限制",
    )?;
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueJson::deserialize(&mut parser)
        .map_err(|e| format!("JSON 格式错误：{e}"))?
        .0;
    parser
        .end()
        .map_err(|e| format!("JSON 尾部内容错误：{e}"))?;
    let decoded: T =
        serde_json::from_value(value.clone()).map_err(|e| format!("数据 schema 错误：{e}"))?;
    let shape = serde_json::to_value(&decoded).map_err(|e| format!("数据序列化错误：{e}"))?;
    fn same_shape(input: &Value, typed: &Value) -> bool {
        match (input, typed) {
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, v)| b.get(k).map_or(false, |expected| same_shape(v, expected)))
            }
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_shape(a, b))
            }
            (Value::Null, Value::Null)
            | (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_)) => true,
            _ => false,
        }
    }
    require(
        same_shape(&value, &shape),
        "数据 schema 含未知字段、缺失字段或不匹配的字段类型",
    )?;
    Ok(decoded)
}

fn redirected(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}
fn check_file(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            require(
                metadata.file_type().is_file() && !redirected(&metadata),
                "存储路径必须为普通文件",
            )?;
            require(
                metadata.len() <= MAX_DOCUMENT_BYTES as u64,
                "存储文件超过 16 MiB 限制",
            )?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("读取存储路径失败：{error}")),
    }
}
fn check_data_dir(root: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(root.join("data")) {
        Ok(metadata) => {
            require(
                metadata.is_dir() && !redirected(&metadata),
                "root/data 必须为普通目录",
            )?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("读取 data 目录失败：{error}")),
    }
}
fn read_valid(path: &Path) -> Result<(PersistedData, Vec<u8>), String> {
    require(check_file(path)?, "存储文件不存在")?;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| format!("打开存储失败：{e}"))?
        .take(MAX_DOCUMENT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("读取存储失败：{e}"))?;
    let data = decode_strict(&bytes)?;
    validate_data(&data)?;
    Ok((data, bytes))
}

/// Returns fixtures only for a genuinely new store. Corruption is surfaced to the UI.
pub fn load(root: &Path) -> Result<PersistedData, String> {
    if !check_data_dir(root)? {
        return Ok(crate::demo::initial_data());
    }
    let directory = root.join("data");
    let current = directory.join(STATE_FILE);
    if !check_file(&current)? {
        for name in [
            PREVIOUS_FILE,
            "state.json.tmp",
            "state.previous.json.tmp",
            ".state.lock",
        ] {
            require(
                !check_file(&directory.join(name))?,
                "当前存储缺失，已保留上次版本或中断现场，请检查 data 目录",
            )?;
        }
        return Ok(crate::demo::initial_data());
    }
    let data = read_valid(&current)
        .map(|v| v.0)
        .map_err(|e| format!("本地数据校验失败（已保留文件）：{e}"))?;
    let previous = directory.join(PREVIOUS_FILE);
    if check_file(&previous)? {
        read_valid(&previous).map_err(|e| format!("上一有效版需要检查（已保留文件）：{e}"))?;
    }
    Ok(data)
}

struct OwnedFile(PathBuf);
impl Drop for OwnedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn write_synced(path: &Path, bytes: &[u8]) -> Result<OwnedFile, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("创建临时文件失败：{e}"))?;
    let guard = OwnedFile(path.into());
    file.write_all(bytes)
        .map_err(|e| format!("写入临时文件失败：{e}"))?;
    file.flush()
        .map_err(|e| format!("flush 临时文件失败：{e}"))?;
    file.sync_all()
        .map_err(|e| format!("sync 临时文件失败：{e}"))?;
    Ok(guard)
}
fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|e| format!("同步 data 目录失败：{e}"))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Validates before touching disk, then flushes/syncs and atomically replaces files.
/// The previous valid revision is retained; damaged files always produce an error.
pub fn save(root: &Path, data: &PersistedData) -> Result<(), String> {
    validate_data(data)?;
    let bytes = serde_json::to_vec_pretty(data).map_err(|e| format!("序列化存储失败：{e}"))?;
    require(
        bytes.len() <= MAX_DOCUMENT_BYTES,
        "存储数据超过 16 MiB 限制",
    )?;
    require(root.is_dir(), "插件 root 目录不存在")?;
    let directory = root.join("data");
    if !check_data_dir(root)? {
        fs::create_dir(&directory).map_err(|e| format!("创建 data 目录失败：{e}"))?;
    }
    check_data_dir(root)?;
    let lock = directory.join(".state.lock");
    let lock_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|e| format!("存储正忙或上次写入中断，请检查 data/.state.lock：{e}"))?;
    let _lock_guard = OwnedFile(lock);
    let current = directory.join(STATE_FILE);
    let previous = directory.join(PREVIOUS_FILE);
    let old = if check_file(&current)? {
        Some(read_valid(&current)?.1)
    } else {
        None
    };
    if check_file(&previous)? {
        read_valid(&previous)?;
        require(old.is_some(), "当前存储缺失；已保留上一有效版，请先恢复")?;
    }
    for name in ["state.json.tmp", "state.previous.json.tmp"] {
        require(
            !directory.join(name).exists(),
            "检测到中断的临时文件，请检查 data 目录后重试",
        )?;
    }
    let temp = write_synced(&directory.join("state.json.tmp"), &bytes)?;
    if let Some(old_bytes) = old {
        let previous_temp = write_synced(&directory.join("state.previous.json.tmp"), &old_bytes)?;
        fs::rename(&previous_temp.0, &previous)
            .map_err(|e| format!("替换上一有效版本失败：{e}"))?;
        sync_directory(&directory)?;
    }
    fs::rename(&temp.0, &current).map_err(|e| format!("原子替换当前存储失败：{e}"))?;
    sync_directory(&directory)?;
    drop(lock_file);
    Ok(())
}
