//! Human-readable backup interchange with deterministic, recursively sorted JSON hashing.
use crate::{contracts::*, store};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportedBackup {
    id: String,
    created_at: i64,
    source: String,
    source_game_version: String,
    device_name: String,
    snapshot: Snapshot,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Integrity {
    algorithm: String,
    digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    format: String,
    format_version: u32,
    plugin_version: String,
    exported_at: String,
    backup: ExportedBackup,
    integrity: Integrity,
}

/// RFC-independent canonicalization used here: recursively sort object keys, retain
/// array order, then serialize minified JSON as UTF-8. No whitespace or BOM is hashed.
pub fn canonical_bytes(value: &Value) -> Result<Vec<u8>, String> {
    fn write(value: &Value, output: &mut Vec<u8>) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                output.push(b'{');
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort_unstable();
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 {
                        output.push(b',');
                    }
                    serde_json::to_writer(&mut *output, key).map_err(|e| e.to_string())?;
                    output.push(b':');
                    write(&map[key], output)?;
                }
                output.push(b'}');
            }
            Value::Array(values) => {
                output.push(b'[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push(b',');
                    }
                    write(value, output)?;
                }
                output.push(b']');
            }
            _ => serde_json::to_writer(output, value).map_err(|e| e.to_string())?,
        }
        Ok(())
    }
    let mut output = Vec::new();
    write(value, &mut output)?;
    Ok(output)
}
fn digest_without_integrity(envelope: &Envelope) -> Result<String, String> {
    let mut value = serde_json::to_value(envelope).map_err(|e| e.to_string())?;
    value
        .as_object_mut()
        .ok_or_else(|| "导出根节点必须是对象".to_string())?
        .remove("integrity");
    let bytes = canonical_bytes(&value)?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|v| format!("{v:02x}")).collect())
}

/// Exports the complete simulated snapshot; the filename is chosen by the runtime.
pub fn export_backup(record: &BackupRecord, exported_at: i64) -> Result<Vec<u8>, String> {
    store::validate_backup(record)?;
    if !(0..=253_402_300_799_999).contains(&exported_at) {
        return Err("exportedAt 超出 Unix 毫秒范围".into());
    }
    if exported_at < record.created_at {
        return Err("exportedAt 早于备份创建时间".into());
    }
    let exported_at = Utc
        .timestamp_millis_opt(exported_at)
        .single()
        .ok_or_else(|| "exportedAt 时间无效".to_string())?
        .to_rfc3339_opts(SecondsFormat::Millis, true);
    let mut envelope = Envelope {
        format: "deltaforce-sync-backup".into(),
        format_version: 1,
        plugin_version: PLUGIN_VERSION.into(),
        exported_at,
        backup: ExportedBackup {
            id: record.id.clone(),
            created_at: record.created_at,
            source: record.source.clone(),
            source_game_version: record.snapshot.source_game_version.clone(),
            device_name: record.device_name.clone(),
            snapshot: record.snapshot.clone(),
        },
        integrity: Integrity {
            algorithm: "sha256".into(),
            digest: String::new(),
        },
    };
    envelope.integrity.digest = digest_without_integrity(&envelope)?;
    let mut bytes =
        serde_json::to_vec_pretty(&envelope).map_err(|e| format!("生成导出 JSON 失败：{e}"))?;
    bytes.push(b'\n');
    if bytes.len() > store::MAX_DOCUMENT_BYTES {
        return Err("导出文件超过 16 MiB 限制".into());
    }
    Ok(bytes)
}

/// Strictly checks schema, metadata, model bounds and hash before yielding a record.
pub fn verify_export(bytes: &[u8]) -> Result<BackupRecord, String> {
    let envelope: Envelope = store::decode_strict(bytes)?;
    if envelope.format != "deltaforce-sync-backup"
        || envelope.format_version != 1
        || envelope.plugin_version != PLUGIN_VERSION
    {
        return Err("导出格式、格式版本或插件版本不受支持".into());
    }
    if envelope.integrity.algorithm != "sha256"
        || envelope.integrity.digest.len() != 64
        || !envelope
            .integrity
            .digest
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
    {
        return Err("integrity 必须包含 sha256 与 64 位小写十六进制 digest".into());
    }
    let exported_at = DateTime::parse_from_rfc3339(&envelope.exported_at)
        .map_err(|e| format!("exportedAt 必须为 RFC3339：{e}"))?
        .timestamp_millis();
    if !(0..=253_402_300_799_999).contains(&exported_at) || exported_at < envelope.backup.created_at
    {
        return Err("exportedAt 超出有效范围或早于备份".into());
    }
    if envelope.backup.source_game_version != envelope.backup.snapshot.source_game_version {
        return Err("备份 sourceGameVersion 与快照不一致".into());
    }
    let digest = digest_without_integrity(&envelope)?;
    // The digest protects local interchange integrity; it is intentionally unkeyed.
    if digest != envelope.integrity.digest {
        return Err("SHA-256 完整性校验失败，文件内容已改变".into());
    }
    let backup = envelope.backup;
    let record = BackupRecord {
        id: backup.id,
        created_at: backup.created_at,
        source: backup.source,
        device_name: backup.device_name,
        snapshot: backup.snapshot,
    };
    store::validate_backup(&record)?;
    Ok(record)
}
