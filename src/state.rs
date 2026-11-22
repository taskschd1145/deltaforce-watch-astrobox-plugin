//! Pure UI state transitions. Persistence/export I/O is performed by the host adapter.
use crate::{contracts::*, store};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

fn sort_snapshot(snapshot: &mut Snapshot) {
    snapshot.matches.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| a.id.cmp(&b.id))
    });
}
fn normalize(data: &mut PersistedData) {
    sort_snapshot(&mut data.snapshot);
    for backup in &mut data.backups {
        sort_snapshot(&mut backup.snapshot);
    }
    data.backups.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.id.cmp(&a.id))
    });
}
fn quality_rank(quality: &str) -> u8 {
    match quality {
        "红" => 6,
        "金" => 5,
        "紫" => 4,
        "蓝" => 3,
        "绿" => 2,
        "白" => 1,
        _ => 0,
    }
}

impl AppState {
    /// Each launch starts disconnected. Automatic backup waits for explicit demo connection.
    pub fn new(mut data: PersistedData) -> AppState {
        normalize(&mut data);
        let notice = match store::validate_data(&data) {
            Ok(()) => "UI Demo · 所有界面数据均为模拟，点击模拟连接读取示例快照。".into(),
            Err(error) => format!("本地模型需要检查：{error}；当前界面为模拟数据展示。"),
        };
        AppState {
            auto_backup_pending: data.settings.auto_backup_on_launch,
            auto_backup_done: false,
            retention_input: data.settings.retention_count.to_string(),
            data,
            page: Page::Matches,
            history_filter: "all".into(),
            selected_match_id: None,
            viewed_backup_id: None,
            services_tab: "inventory".into(),
            inventory_query: String::new(),
            inventory_quality: "all".into(),
            inventory_category: "all".into(),
            mission_filter: "all".into(),
            selected_item_id: None,
            connected: false,
            busy: false,
            notice,
            confirmation: None,
            timezone_offset_minutes: 480,
        }
    }
    /// Preview is purely ephemeral and never replaces the persisted active snapshot.
    pub fn display_snapshot(&self) -> &Snapshot {
        self.viewed_backup_id
            .as_ref()
            .and_then(|id| self.data.backups.iter().find(|backup| &backup.id == id))
            .map(|backup| &backup.snapshot)
            .unwrap_or(&self.data.snapshot)
    }
    pub fn filtered_matches(&self) -> Vec<&MatchRecord> {
        let mut records: Vec<_> = self
            .display_snapshot()
            .matches
            .iter()
            .filter(|record| self.history_filter == "all" || record.mode == self.history_filter)
            .collect();
        records.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        records
    }
    pub fn selected_match(&self) -> Option<&MatchRecord> {
        self.selected_match_id.as_ref().and_then(|id| {
            self.display_snapshot()
                .matches
                .iter()
                .find(|record| &record.id == id)
        })
    }
    pub fn filtered_inventory(&self) -> Vec<&Item> {
        let query = self.inventory_query.trim().to_lowercase();
        let mut items: Vec<_> = self
            .display_snapshot()
            .inventory
            .iter()
            .filter(|item| {
                (self.inventory_quality == "all" || item.quality == self.inventory_quality)
                    && (self.inventory_category == "all"
                        || item.category == self.inventory_category)
                    && (query.is_empty()
                        || item.name.to_lowercase().contains(&query)
                        || item.id.to_lowercase().contains(&query))
            })
            .collect();
        items.sort_by(|a, b| {
            quality_rank(&b.quality)
                .cmp(&quality_rank(&a.quality))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.id.cmp(&b.id))
        });
        items
    }
    pub fn selected_item(&self) -> Option<&Item> {
        let id = self.selected_item_id.as_ref()?;
        let snapshot = self.display_snapshot();
        snapshot
            .inventory
            .iter()
            .chain(
                snapshot
                    .matches
                    .iter()
                    .flat_map(|record| record.loot.iter()),
            )
            .find(|item| &item.id == id)
    }
    pub fn filtered_missions(&self) -> Vec<&Mission> {
        self.display_snapshot()
            .missions
            .iter()
            .filter(|mission| match self.mission_filter.as_str() {
                "pending" => mission.progress < mission.target,
                "completed" => mission.progress >= mission.target,
                _ => true,
            })
            .collect()
    }
}

pub fn money(value: u64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    output
}

pub fn format_time(ms: Option<i64>, offset: i32) -> String {
    let Some(ms) = ms else {
        return "时间未知".into();
    };
    let Some(seconds) = offset.checked_mul(60) else {
        return "时间无效".into();
    };
    if FixedOffset::east_opt(seconds).is_none() {
        return "时区无效".into();
    }
    let Some(time) = DateTime::from_timestamp_millis(ms) else {
        return "时间无效".into();
    };
    let Some(local) = time
        .naive_utc()
        .checked_add_signed(chrono::Duration::seconds(i64::from(seconds)))
    else {
        return "时间无效".into();
    };
    local.format("%Y-%m-%d %H:%M").to_string()
}

pub fn outcome_label(record: &MatchRecord) -> &'static str {
    match record.outcome.as_str() {
        "extracted" => "撤离成功",
        "killed" => "撤离失败",
        "timeout" => "撤离失败 · 超时",
        "victory" => "胜利",
        "defeat" => "失败",
        "draw" => "平局",
        "practice" => "训练",
        "aborted" => "已中止",
        _ => "未知结果",
    }
}

fn input_text(payload: &str, limit: usize) -> Result<String, String> {
    let trimmed = payload.trim();
    let value = match serde_json::from_str::<Value>(trimmed) {
        Ok(Value::String(value)) => value,
        Ok(Value::Object(object)) => object
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| "输入事件需要字符串 value 字段".to_string())?
            .to_string(),
        Ok(Value::Array(_)) => return Err("输入事件需要文本或含 value 的对象".into()),
        Ok(Value::Number(_)) | Ok(Value::Bool(_)) | Ok(Value::Null) => trimmed.into(),
        Err(_)
            if trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.starts_with('"') =>
        {
            return Err("输入事件 JSON 格式错误".into());
        }
        Err(_) => payload.into(),
    };
    if value.chars().count() > limit || value.chars().any(char::is_control) {
        return Err(format!("输入最多 {limit} 个有效字符"));
    }
    Ok(value)
}
fn persist() -> Effect {
    Effect {
        persist: true,
        export_backup: None,
    }
}
fn clear_selection(s: &mut AppState) {
    s.selected_match_id = None;
    s.selected_item_id = None;
}
fn cleanup_preview(s: &mut AppState) {
    if s.viewed_backup_id
        .as_ref()
        .is_some_and(|id| !s.data.backups.iter().any(|backup| &backup.id == id))
    {
        s.viewed_backup_id = None;
        clear_selection(s);
    }
}
fn create_backup(s: &mut AppState, now: i64) -> Result<String, String> {
    if !(0..=253_402_300_799_999).contains(&now)
        || now < s.data.snapshot.captured_at
        || s.data.backups.iter().any(|backup| now < backup.created_at)
    {
        return Err("当前时间无效或早于已有模拟数据，请检查宿主时间".into());
    }
    let prefix = format!("backup-{now}-");
    let last = s
        .data
        .backups
        .iter()
        .filter_map(|backup| {
            backup
                .id
                .strip_prefix(&prefix)
                .and_then(|suffix| suffix.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    let sequence = last
        .checked_add(1)
        .ok_or_else(|| "备份序号超出范围".to_string())?;
    let id = format!("{prefix}{sequence:010}");
    let record = BackupRecord {
        id: id.clone(),
        created_at: now,
        source: "mock".into(),
        device_name: "演示手表 · 模拟设备".into(),
        snapshot: s.data.snapshot.clone(),
    };
    store::validate_backup(&record)?;
    s.data.backups.push(record);
    normalize(&mut s.data);
    s.data
        .backups
        .truncate(s.data.settings.retention_count as usize);
    cleanup_preview(s);
    Ok(id)
}
fn existing_backup<'a>(s: &'a AppState, id: &str) -> Result<&'a BackupRecord, String> {
    s.data
        .backups
        .iter()
        .find(|backup| backup.id == id)
        .ok_or_else(|| "备份已不存在，请刷新列表".into())
}
fn set_retention(s: &mut AppState, count: u32) -> Result<Effect, String> {
    if !(1..=50).contains(&count) {
        return Err("备份保留数量必须为 1..50 的整数".into());
    }
    let before = s.data.backups.len();
    normalize(&mut s.data);
    s.data.settings.retention_count = count;
    s.data.backups.truncate(count as usize);
    s.retention_input = count.to_string();
    cleanup_preview(s);
    s.notice = format!(
        "保留策略已设为 {count} 份模拟备份，清理 {} 份旧备份。",
        before - s.data.backups.len()
    );
    Ok(persist())
}

fn dispatch_inner(s: &mut AppState, id: &str, payload: &str, now: i64) -> Result<Effect, String> {
    if s.busy
        && (id.starts_with("backup:")
            || id.starts_with("settings:")
            || id.starts_with("confirm:")
            || id == "device:toggle")
    {
        return Err("正在导出模拟备份，请等待当前操作完成。".into());
    }
    if s.confirmation.is_some() && !["confirm:accept", "confirm:cancel"].contains(&id) {
        return Err("请先确认或取消当前操作。".into());
    }
    match id {
        "nav:matches" | "back:matches" => {
            s.page = Page::Matches;
            clear_selection(s);
        }
        "nav:services" => {
            s.page = Page::Services;
            s.selected_item_id = None;
        }
        "nav:settings" => {
            s.page = Page::Settings;
            s.selected_item_id = None;
        }
        "preview:exit" => {
            s.viewed_backup_id = None;
            clear_selection(s);
            s.page = Page::Matches;
            s.notice = "已回到当前模拟快照。".into();
        }
        "filter:all" | "filter:operations" | "filter:battlefield" => {
            s.history_filter = id.trim_start_matches("filter:").into();
            s.page = Page::Matches;
            s.selected_match_id = None;
        }
        "services:inventory" | "services:facilities" | "services:missions" => {
            s.services_tab = id.trim_start_matches("services:").into();
            s.page = Page::Services;
            s.selected_item_id = None;
        }
        "query:inventory" => {
            s.inventory_query = input_text(payload, 120)?;
        }
        "quality:all" | "quality:白" | "quality:绿" | "quality:蓝" | "quality:紫"
        | "quality:金" | "quality:红" => {
            s.inventory_quality = id.trim_start_matches("quality:").into();
        }
        "category:all" | "category:装备" | "category:弹药" | "category:药品"
        | "category:贵重品" | "category:房卡" => {
            s.inventory_category = id.trim_start_matches("category:").into();
        }
        "mission:all" | "mission:pending" | "mission:completed" => {
            s.mission_filter = id.trim_start_matches("mission:").into();
        }
        "item:close" => {
            s.selected_item_id = None;
        }
        "device:toggle" => {
            if s.connected {
                s.connected = false;
                s.notice = "模拟连接已断开，已读取的示例快照保留在本地。".into();
            } else {
                s.connected = true;
                s.data.snapshot = crate::demo::initial_data().snapshot;
                s.viewed_backup_id = None;
                clear_selection(s);
                if s.auto_backup_pending && !s.auto_backup_done {
                    create_backup(s, now)?;
                    s.auto_backup_pending = false;
                    s.auto_backup_done = true;
                    s.notice = "模拟连接成功，示例快照已读取；本次启动的自动备份已完成。".into();
                } else {
                    s.notice = "模拟连接成功，已读取示例快照；所有界面数据均为模拟。".into();
                }
                return Ok(persist());
            }
        }
        "backup:create" => {
            if !s.connected {
                return Err("请先点击模拟连接，再创建模拟备份。".into());
            }
            let id = create_backup(s, now)?;
            s.notice = format!("已创建当前模拟快照备份：{id}。");
            return Ok(persist());
        }
        "settings:auto-backup" => {
            s.data.settings.auto_backup_on_launch = !s.data.settings.auto_backup_on_launch;
            s.auto_backup_pending = s.data.settings.auto_backup_on_launch && !s.auto_backup_done;
            s.notice = if s.data.settings.auto_backup_on_launch {
                "已开启启动自动备份，等待显式模拟连接；每次启动最多自动备份一次。"
            } else {
                "已关闭启动自动备份。"
            }
            .into();
            return Ok(persist());
        }
        "settings:retention-input" => {
            s.retention_input = input_text(payload, 32)?;
        }
        "settings:retention-apply" => {
            let value = s.retention_input.trim();
            if value.is_empty() || !value.bytes().all(|v| v.is_ascii_digit()) {
                return Err("备份保留数量必须为 1..50 的整数".into());
            }
            let count = value
                .parse::<u32>()
                .map_err(|_| "备份保留数量必须为 1..50 的整数".to_string())?;
            if !(1..=50).contains(&count) {
                return Err("备份保留数量必须为 1..50 的整数".into());
            }
            if (count as usize) < s.data.backups.len() {
                s.confirmation = Some(Confirmation::SetRetention(count));
                s.notice = format!(
                    "调整为 {count} 份将清理 {} 份最旧模拟备份，请确认。",
                    s.data.backups.len() - count as usize
                );
            } else if s.data.settings.retention_count != count {
                return set_retention(s, count);
            } else {
                s.retention_input = count.to_string();
                s.notice = "当前保留策略已生效。".into();
            }
        }
        "confirm:cancel" => {
            s.confirmation = None;
            s.notice = "已取消操作，模拟数据保持原样。".into();
        }
        "confirm:accept" => {
            let confirmation = s
                .confirmation
                .take()
                .ok_or_else(|| "当前没有待确认的操作".to_string())?;
            match confirmation {
                Confirmation::Restore(id) => {
                    let snapshot = existing_backup(s, &id)?.snapshot.clone();
                    store::validate_snapshot(&snapshot)?;
                    s.data.snapshot = snapshot;
                    s.viewed_backup_id = None;
                    clear_selection(s);
                    s.page = Page::Matches;
                    s.notice = format!("已恢复本地模拟快照：{id}。");
                }
                Confirmation::DeleteBackup(id) => {
                    existing_backup(s, &id)?;
                    s.data.backups.retain(|backup| backup.id != id);
                    cleanup_preview(s);
                    s.notice = format!("已删除模拟备份：{id}。");
                }
                Confirmation::SetRetention(count) => return set_retention(s, count),
            }
            return Ok(persist());
        }
        _ => {
            if let Some(match_id) = id.strip_prefix("match:") {
                if !s
                    .display_snapshot()
                    .matches
                    .iter()
                    .any(|record| record.id == match_id)
                {
                    return Err("对局已不存在，请返回战绩列表".into());
                }
                s.selected_match_id = Some(match_id.into());
                s.page = Page::MatchDetail;
                s.selected_item_id = None;
            } else if let Some(item_id) = id.strip_prefix("item:") {
                let snapshot = s.display_snapshot();
                if !snapshot
                    .inventory
                    .iter()
                    .chain(
                        snapshot
                            .matches
                            .iter()
                            .flat_map(|record| record.loot.iter()),
                    )
                    .any(|item| item.id == item_id)
                {
                    return Err("物资已不存在，请刷新列表".into());
                }
                s.selected_item_id = Some(item_id.into());
            } else if let Some(backup_id) = id.strip_prefix("backup:view:") {
                existing_backup(s, backup_id)?;
                s.viewed_backup_id = Some(backup_id.into());
                s.page = Page::Matches;
                clear_selection(s);
                s.notice = format!("正在预览模拟备份 {backup_id}；当前快照保持原样。");
            } else if let Some(backup_id) = id.strip_prefix("backup:restore:") {
                existing_backup(s, backup_id)?;
                s.confirmation = Some(Confirmation::Restore(backup_id.into()));
                s.notice = "恢复将替换当前本地模拟快照，请确认。".into();
            } else if let Some(backup_id) = id.strip_prefix("backup:delete:") {
                existing_backup(s, backup_id)?;
                s.confirmation = Some(Confirmation::DeleteBackup(backup_id.into()));
                s.notice = "将删除这份模拟备份，请确认。".into();
            } else if let Some(backup_id) = id.strip_prefix("backup:export:") {
                store::validate_backup(existing_backup(s, backup_id)?)?;
                s.busy = true;
                s.notice = "正在导出并校验模拟备份…".into();
                return Ok(Effect {
                    persist: false,
                    export_backup: Some(backup_id.into()),
                });
            } else {
                return Err(format!("当前 UI Demo 未识别事件：{id}"));
            }
        }
    }
    Ok(Effect::default())
}

/// Transactional, error-safe dispatcher. A failed event changes only the notice.
/// The runtime clones the returned state again when persisting and commits it only
/// after store::save succeeds, preserving the previous retention policy on I/O errors.
pub fn dispatch(s: &mut AppState, id: &str, payload: &str, now: i64) -> Effect {
    if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) || payload.len() > 8_192
    {
        s.notice = "事件 ID 或载荷超过边界，请重试。".into();
        return Effect::default();
    }
    let mut candidate = s.clone();
    match dispatch_inner(&mut candidate, id, payload, now).and_then(|effect| {
        if effect.persist {
            store::validate_data(&candidate.data)?;
        }
        Ok(effect)
    }) {
        Ok(effect) => {
            *s = candidate;
            effect
        }
        Err(error) => {
            s.notice = format!("操作提示：{error}");
            Effect::default()
        }
    }
}
