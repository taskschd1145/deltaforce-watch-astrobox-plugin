use deltaforce_watch_astrobox_plugin::{contracts::*, demo, export, state, store};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}};

const NOW: i64 = DEMO_CAPTURED_AT + 60_000;
fn app() -> AppState { AppState::new(demo::initial_data()) }
fn send(s: &mut AppState, id: &str) -> Effect { state::dispatch(s, id, "", NOW) }
fn retention(s: &mut AppState, value: &str) -> Effect {
    state::dispatch(s, "settings:retention-input", value, NOW);
    send(s, "settings:retention-apply")
}
struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!("deltaforce-sync-core-{}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str) -> PathBuf { self.0.join("data").join(name) }
}
impl Drop for TempRoot { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test]
fn complete_fixtures_are_explicit_mock_and_valid() {
    let data = demo::initial_data();
    store::validate_data(&data).unwrap();
    let snapshot = &data.snapshot;
    assert_eq!(snapshot.source, "mock");
    assert_eq!(snapshot.source_game_version, "1.2.2.4");
    assert_eq!(snapshot.matches.len(), 12);
    assert_eq!(snapshot.matches.iter().filter(|m| m.mode == "operations").count(), 8);
    assert_eq!(snapshot.matches.iter().filter(|m| m.mode == "battlefield").count(), 4);
    for outcome in ["extracted", "killed", "timeout", "victory", "defeat", "draw", "practice"] {
        assert!(snapshot.matches.iter().any(|m| m.outcome == outcome));
    }
    assert!(snapshot.matches.iter().all(|m| m.started_at.is_some_and(|ms| ms > 1_000_000_000_000)));
    assert_eq!(snapshot.inventory.len(), 20);
    for quality in ["白", "绿", "蓝", "紫", "金", "红"] { assert!(snapshot.inventory.iter().any(|i| i.quality == quality)); }
    assert!(snapshot.inventory.iter().any(|i| i.locked));
    assert!(snapshot.inventory.iter().any(|i| !i.locked));
    assert!(snapshot.inventory.iter().any(|i| i.durability.zip(i.max_durability).is_some_and(|(v, max)| v < max)));
    assert_eq!(snapshot.facilities.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), vec!["仓库", "训练中心"]);
    assert_eq!(snapshot.missions.len(), 6);
    assert!(snapshot.missions.iter().all(|m| m.source == "demo-extension" && m.description.contains("demo-extension")));
    assert_eq!(data.backups.len(), 3);
    assert_ne!(data.backups[0].snapshot.player.funds, data.backups[1].snapshot.player.funds);
    assert_ne!(data.backups[1].snapshot.inventory, data.backups[2].snapshot.inventory);
    assert_eq!(data.settings.retention_count, 5);
    assert!(!data.settings.auto_backup_on_launch);
    let s = AppState::new(data);
    assert!(!s.connected);
    assert!(s.notice.contains("所有界面数据均为模拟"));
}

#[test]
fn histories_and_backups_are_sorted_newest_first_with_missing_time_last() {
    let mut data = demo::initial_data();
    data.snapshot.matches.reverse();
    data.snapshot.matches[0].started_at = None;
    data.backups.reverse();
    let s = AppState::new(data);
    let records = s.filtered_matches();
    assert!(records.windows(2).all(|pair| pair[0].started_at >= pair[1].started_at));
    assert_eq!(records.last().unwrap().started_at, None);
    assert!(s.data.backups.windows(2).all(|pair| pair[0].created_at >= pair[1].created_at));
}

#[test]
fn failed_operations_retain_real_safe_loot_and_score_is_distinct_from_xp() {
    let data = demo::initial_data();
    for record in &data.snapshot.matches {
        let total: u64 = record.loot.iter().map(|item| item.unit_value * u64::from(item.quantity)).sum();
        assert_eq!(record.loot_value, total);
        assert_ne!(record.score, record.xp_gain);
        if ["killed", "timeout"].contains(&record.outcome.as_str()) {
            assert!(record.loot_value > 0);
            assert!(record.loot.iter().all(|item| item.compartment == "safe"));
        }
    }
}

#[test]
fn cumulative_damage_is_independent_of_final_injury_and_can_exceed_one_hundred() {
    let mut data = demo::initial_data();
    assert!(data.snapshot.matches.iter().flat_map(|m| &m.damage).any(|d| d.damage_taken.unwrap_or(0) > 100));
    assert!(data.snapshot.matches.iter().flat_map(|m| &m.damage).any(|d| d.damage_taken.unwrap_or(0) > 0 && d.injury_at_end == Some(0)));
    data.snapshot.matches[0].damage[0].damage_taken = Some(900);
    data.snapshot.matches[0].damage[0].injury_at_end = Some(0);
    data.snapshot.matches[0].damage[1].damage_taken = None;
    data.snapshot.matches[0].damage[1].injury_at_end = Some(40);
    store::validate_data(&data).unwrap();
    data.snapshot.matches[0].damage[0].injury_at_end = Some(101);
    assert!(store::validate_data(&data).is_err());
}

#[test]
fn item_boundaries_and_integral_overflow_are_checked() {
    let data = demo::initial_data();
    let item = data.snapshot.inventory[0].clone();
    let mut invalid = item.clone(); invalid.quantity = 0; assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.slots = 0; assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.max_durability = None; assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.durability = Some(101); assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.quality = "橙".into(); assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.category = "未知".into(); assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.name = "bad\nname".into(); assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.id = "../outside".into(); assert!(store::validate_item(&invalid).is_err());
    let mut invalid = item.clone(); invalid.quantity = 2; invalid.unit_value = u64::MAX; assert!(store::validate_item(&invalid).is_err());
    let mut valid = item; valid.unit_value = u64::MAX; valid.quantity = 1; store::validate_item(&valid).unwrap();
}

#[test]
fn model_boundaries_reject_bad_versions_counts_duplicates_and_totals() {
    let mut data = demo::initial_data(); data.schema_version = 2; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.settings.retention_count = 0; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.settings.retention_count = 51; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.player.warehouse_used += 1; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.missions[0].progress = data.snapshot.missions[0].target + 1; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.matches[0].loot_value += 1; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.matches[0].damage[1].part = "head".into(); assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.matches[0].mode = "battlefield".into(); assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.matches[0].started_at = Some(i64::MAX); assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.captured_at = -1; assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.backups[1].id = data.backups[0].id.clone(); assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.inventory[1].id = data.snapshot.inventory[0].id.clone(); assert!(store::validate_data(&data).is_err());
    let mut data = demo::initial_data(); data.snapshot.missions[0].source = "game".into(); assert!(store::validate_data(&data).is_err());
}

#[test]
fn navigation_filters_search_and_item_detail_follow_the_public_ids() {
    let mut s = app();
    send(&mut s, "filter:operations"); assert_eq!(s.filtered_matches().len(), 8);
    send(&mut s, "filter:battlefield"); assert_eq!(s.filtered_matches().len(), 4);
    send(&mut s, "filter:all");
    send(&mut s, "match:op-008"); assert_eq!(s.page, Page::MatchDetail); assert_eq!(s.selected_match().unwrap().id, "op-008");
    send(&mut s, "back:matches"); assert_eq!(s.page, Page::Matches); assert!(s.selected_match().is_none());
    send(&mut s, "nav:services"); assert_eq!(s.page, Page::Services);
    state::dispatch(&mut s, "query:inventory", r#"{"value":"m4A1"}"#, NOW);
    send(&mut s, "quality:紫"); send(&mut s, "category:装备");
    assert_eq!(s.filtered_inventory().len(), 1);
    assert_eq!(s.filtered_inventory()[0].id, "inv-m4a1");
    send(&mut s, "item:inv-m4a1"); assert_eq!(s.selected_item().unwrap().name, "M4A1 突击步枪");
    send(&mut s, "item:close"); assert!(s.selected_item().is_none());
    state::dispatch(&mut s, "query:inventory", "金条", NOW);
    send(&mut s, "quality:all"); send(&mut s, "category:all"); assert_eq!(s.filtered_inventory().len(), 1);
    send(&mut s, "services:missions"); send(&mut s, "mission:completed"); assert_eq!(s.filtered_missions().len(), 2);
    send(&mut s, "mission:pending"); assert_eq!(s.filtered_missions().len(), 4);
    send(&mut s, "services:facilities"); assert_eq!(s.services_tab, "facilities");
    send(&mut s, "nav:settings"); assert_eq!(s.page, Page::Settings);
}

#[test]
fn malformed_events_are_safe_and_leave_model_and_selection_intact() {
    let mut s = app();
    let before = s.data.clone();
    for (id, payload) in [("match:missing", ""), ("item:missing", ""), ("backup:delete:missing", ""), ("unknown", ""), ("query:inventory", "{broken"), ("query:inventory", r#"{"value":42}"#), ("query:inventory", "line\nfeed")] {
        assert_eq!(state::dispatch(&mut s, id, payload, i64::MAX), Effect::default());
        assert_eq!(s.data, before);
        assert!(s.notice.starts_with("操作提示"));
    }
    assert_eq!(state::dispatch(&mut s, "query:inventory", &"x".repeat(8_193), NOW), Effect::default());
    assert_eq!(s.inventory_query, "");
    assert!(s.selected_item_id.is_none());
}

#[test]
fn manual_backup_requires_connection_and_keeps_newest_unique_revisions() {
    let mut s = app();
    assert!(!send(&mut s, "backup:create").persist); assert_eq!(s.data.backups.len(), 3);
    assert!(send(&mut s, "device:toggle").persist); assert!(s.connected);
    for _ in 0..7 { assert!(send(&mut s, "backup:create").persist); }
    assert_eq!(s.data.backups.len(), 5);
    assert!(s.data.backups.iter().all(|b| b.id.starts_with("backup-") && b.created_at == NOW));
    assert_eq!(s.data.backups.iter().map(|b| &b.id).collect::<std::collections::HashSet<_>>().len(), 5);
    assert!(s.data.backups.windows(2).all(|b| b[0].id > b[1].id));
    store::validate_data(&s.data).unwrap();
}

#[test]
fn backup_view_is_ephemeral_and_restore_requires_confirmation() {
    let mut s = app();
    let original = s.data.clone();
    let selected = s.data.backups[2].clone();
    let id = selected.id.clone();
    assert!(!send(&mut s, &format!("backup:view:{id}")).persist);
    assert_eq!(s.page, Page::Matches); assert_eq!(s.data, original);
    assert_eq!(s.display_snapshot(), &selected.snapshot);
    send(&mut s, "preview:exit"); assert_eq!(s.display_snapshot(), &original.snapshot);
    send(&mut s, &format!("backup:restore:{id}"));
    assert_eq!(s.confirmation, Some(Confirmation::Restore(id.clone()))); assert_eq!(s.data, original);
    send(&mut s, "confirm:cancel"); assert_eq!(s.data, original);
    send(&mut s, &format!("backup:view:{id}"));
    send(&mut s, &format!("backup:restore:{id}"));
    assert!(send(&mut s, "confirm:accept").persist);
    assert_eq!(s.data.snapshot, selected.snapshot);
    assert_eq!(s.data.backups, original.backups);
    assert_eq!(s.data.settings, original.settings);
    assert!(s.viewed_backup_id.is_none());
}

#[test]
fn delete_and_retention_cleanup_are_confirmed_and_keep_newest() {
    let mut s = app();
    let oldest = s.data.backups[2].id.clone();
    send(&mut s, &format!("backup:delete:{oldest}")); send(&mut s, "confirm:cancel"); assert_eq!(s.data.backups.len(), 3);
    send(&mut s, &format!("backup:delete:{oldest}")); assert!(send(&mut s, "confirm:accept").persist); assert_eq!(s.data.backups.len(), 2);
    let newest = s.data.backups[0].id.clone();
    assert!(!retention(&mut s, "1").persist);
    assert_eq!(s.confirmation, Some(Confirmation::SetRetention(1)));
    assert_eq!(s.data.settings.retention_count, 5); assert_eq!(s.data.backups.len(), 2);
    send(&mut s, "confirm:cancel"); assert_eq!(s.data.backups.len(), 2);
    retention(&mut s, "1"); assert!(send(&mut s, "confirm:accept").persist);
    assert_eq!(s.data.settings.retention_count, 1); assert_eq!(s.data.backups.len(), 1); assert_eq!(s.data.backups[0].id, newest);
    assert!(retention(&mut s, "50").persist); assert_eq!(s.data.settings.retention_count, 50);
}

#[test]
fn retention_field_rejects_non_integral_and_out_of_range_values() {
    let mut s = app();
    for value in ["0", "51", "-1", "1.5", "1e1", "", "999999999999999999999", "abc"] {
        let data = s.data.clone();
        assert!(!retention(&mut s, value).persist);
        assert_eq!(s.data, data); assert!(s.confirmation.is_none());
    }
    assert!(retention(&mut s, r#"{"value":"6"}"#).persist);
    assert_eq!(s.data.settings.retention_count, 6);
}

#[test]
fn automatic_backup_is_once_per_launch_and_waits_for_explicit_connection() {
    let mut data = demo::initial_data(); data.settings.auto_backup_on_launch = true;
    let mut s = AppState::new(data);
    assert!(!s.connected); assert!(s.auto_backup_pending); assert!(!s.auto_backup_done); assert_eq!(s.data.backups.len(), 3);
    assert!(send(&mut s, "device:toggle").persist);
    assert!(s.auto_backup_done); assert!(!s.auto_backup_pending); assert_eq!(s.data.backups.len(), 4);
    assert!(!send(&mut s, "device:toggle").persist);
    assert!(send(&mut s, "device:toggle").persist); assert_eq!(s.data.backups.len(), 4);
    send(&mut s, "settings:auto-backup"); send(&mut s, "settings:auto-backup");
    assert!(s.data.settings.auto_backup_on_launch); assert!(!s.auto_backup_pending);
    send(&mut s, "device:toggle"); send(&mut s, "device:toggle"); assert_eq!(s.data.backups.len(), 4);
    let relaunched = AppState::new(s.data);
    assert!(!relaunched.connected); assert!(relaunched.auto_backup_pending); assert!(!relaunched.auto_backup_done);
}

#[test]
fn export_effect_is_single_flight_and_never_persists_busy() {
    let mut s = app();
    let before = s.data.clone();
    let id = s.data.backups[0].id.clone();
    let effect = send(&mut s, &format!("backup:export:{id}"));
    assert_eq!(effect, Effect { persist: false, export_backup: Some(id.clone()) }); assert!(s.busy);
    assert_eq!(send(&mut s, &format!("backup:export:{id}")), Effect::default());
    assert_eq!(send(&mut s, "settings:auto-backup"), Effect::default());
    assert_eq!(s.data, before);
    // The runtime clears this ephemeral latch in its completion/error path.
    s.busy = false;
    assert!(send(&mut s, &format!("backup:export:{id}")).export_backup.is_some());
}

#[test]
fn clocks_and_missing_targets_fail_transactionally() {
    let mut s = app(); send(&mut s, "device:toggle");
    let before = s.data.clone();
    assert!(!state::dispatch(&mut s, "backup:create", "", -1).persist);
    assert!(!state::dispatch(&mut s, "backup:create", "", i64::MAX).persist);
    assert_eq!(s.data, before);
    s.confirmation = Some(Confirmation::Restore("missing".into()));
    assert!(!send(&mut s, "confirm:accept").persist);
    assert_eq!(s.data, before); assert!(s.confirmation.is_some());
}

#[test]
fn number_and_time_formatters_cover_boundaries() {
    assert_eq!(state::money(0), "0"); assert_eq!(state::money(999), "999"); assert_eq!(state::money(1_000), "1,000");
    assert_eq!(state::money(u64::MAX), "18,446,744,073,709,551,615");
    assert_eq!(state::format_time(Some(0), 480), "1970-01-01 08:00");
    assert_eq!(state::format_time(Some(0), -480), "1969-12-31 16:00");
    assert_eq!(state::format_time(None, 480), "时间未知");
    assert_eq!(state::format_time(Some(i64::MAX), 0), "时间无效");
    assert_eq!(state::format_time(Some(0), i32::MAX), "时间无效");
    assert_eq!(state::format_time(Some(0), 1_440), "时区无效");
    for record in demo::initial_data().snapshot.matches { assert_ne!(state::outcome_label(&record), "未知结果"); }
}

fn rehash(value: &mut Value) {
    let mut unsigned = value.clone();
    unsigned.as_object_mut().unwrap().remove("integrity");
    let digest = Sha256::digest(export::canonical_bytes(&unsigned).unwrap());
    value["integrity"]["digest"] = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>().into();
}
fn exported() -> Value { serde_json::from_slice(&export::export_backup(&demo::initial_data().backups[0], NOW).unwrap()).unwrap() }

#[test]
fn canonicalization_recursively_sorts_keys_and_preserves_array_order() {
    let value = json!({"z": [{"z": 2, "a": "中文"}, 1], "a": {"y": false, "b": null}});
    assert_eq!(String::from_utf8(export::canonical_bytes(&value).unwrap()).unwrap(), r#"{"a":{"b":null,"y":false},"z":[{"a":"中文","z":2},1]}"#);
}

#[test]
fn exports_are_readable_utf8_and_roundtrip_exactly() {
    for record in demo::initial_data().backups {
        let bytes = export::export_backup(&record, NOW).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("三角洲演示干员")); assert!(text.contains("\n  \"format\""));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["format"], "deltaforce-sync-backup"); assert_eq!(value["formatVersion"], 1); assert_eq!(value["pluginVersion"], "0.1.0");
        assert_eq!(value["backup"]["sourceGameVersion"], "1.2.2.4"); assert!(value["exportedAt"].as_str().unwrap().ends_with('Z'));
        assert_eq!(export::verify_export(&bytes).unwrap(), record);
        let compact = serde_json::to_vec(&value).unwrap(); assert_eq!(export::verify_export(&compact).unwrap(), record);
        let mut expected = value.clone(); rehash(&mut expected); assert_eq!(expected["integrity"], value["integrity"]);
    }
}

#[test]
fn export_detects_tampering_bad_hashes_and_invalid_utf8() {
    let mut value = exported(); value["backup"]["snapshot"]["player"]["funds"] = json!(42);
    assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["integrity"]["digest"] = "0".repeat(64).into(); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["integrity"]["algorithm"] = "md5".into(); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(export::verify_export(&[0xff, 0xfe]).is_err());
    assert!(export::export_backup(&demo::initial_data().backups[0], -1).is_err());
    assert!(export::export_backup(&demo::initial_data().backups[0], 0).is_err());
}

#[test]
fn export_rejects_invalid_schema_even_with_a_recomputed_digest() {
    let mut value = exported(); value["formatVersion"] = json!(2); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["extra"] = true.into(); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["backup"]["snapshot"]["player"]["extra"] = true.into(); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["backup"]["sourceGameVersion"] = "0.0".into(); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["backup"]["snapshot"]["matches"][0]["lootValue"] = 1.into(); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["backup"]["snapshot"]["inventory"][0].as_object_mut().unwrap().remove("durability"); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = exported(); value["exportedAt"] = "not-a-time".into(); rehash(&mut value); assert!(export::verify_export(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn duplicate_json_keys_and_missing_optional_fields_are_rejected() {
    let value = exported();
    let text = serde_json::to_string(&value).unwrap();
    let duplicate = text.replacen("\"formatVersion\":1", "\"formatVersion\":1,\"formatVersion\":1", 1);
    assert_ne!(text, duplicate); assert!(export::verify_export(duplicate.as_bytes()).is_err());
    let mut value = serde_json::to_value(demo::initial_data()).unwrap();
    value["snapshot"]["matches"][0].as_object_mut().unwrap().remove("startedAt");
    assert!(store::decode_strict::<PersistedData>(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn new_store_and_three_revisions_roundtrip_and_retain_previous_valid_bytes() {
    let root = TempRoot::new();
    let mut data = demo::initial_data();
    assert_eq!(store::load(&root.0).unwrap(), data); assert!(!root.0.join("data").exists());
    store::save(&root.0, &data).unwrap(); assert_eq!(store::load(&root.0).unwrap(), data);
    let original = fs::read(root.file(store::STATE_FILE)).unwrap();
    data.snapshot.player.funds += 100; store::save(&root.0, &data).unwrap();
    assert_eq!(fs::read(root.file(store::PREVIOUS_FILE)).unwrap(), original);
    let second = fs::read(root.file(store::STATE_FILE)).unwrap();
    data.snapshot.player.funds += 100; store::save(&root.0, &data).unwrap();
    assert_eq!(fs::read(root.file(store::PREVIOUS_FILE)).unwrap(), second);
    assert_eq!(store::load(&root.0).unwrap(), data);
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
    assert_eq!(fs::read_dir(root.0.join("data")).unwrap().count(), 2);
}

#[test]
fn corrupt_current_is_reported_and_preserved_by_load_and_save() {
    let root = TempRoot::new(); let mut data = demo::initial_data();
    store::save(&root.0, &data).unwrap(); data.snapshot.player.funds += 1; store::save(&root.0, &data).unwrap();
    let previous = fs::read(root.file(store::PREVIOUS_FILE)).unwrap();
    fs::write(root.file(store::STATE_FILE), b"{broken").unwrap();
    assert!(store::load(&root.0).is_err()); assert!(store::save(&root.0, &data).is_err());
    assert_eq!(fs::read(root.file(store::STATE_FILE)).unwrap(), b"{broken");
    assert_eq!(fs::read(root.file(store::PREVIOUS_FILE)).unwrap(), previous);
}

#[test]
fn missing_current_and_corrupt_previous_require_explicit_recovery() {
    let root = TempRoot::new(); let data = demo::initial_data();
    store::save(&root.0, &data).unwrap(); store::save(&root.0, &data).unwrap();
    fs::remove_file(root.file(store::STATE_FILE)).unwrap();
    assert!(store::load(&root.0).is_err()); assert!(store::save(&root.0, &data).is_err());
    assert!(root.file(store::PREVIOUS_FILE).exists());
    let root = TempRoot::new(); store::save(&root.0, &data).unwrap();
    fs::write(root.file(store::PREVIOUS_FILE), b"bad previous").unwrap();
    let current = fs::read(root.file(store::STATE_FILE)).unwrap();
    assert!(store::load(&root.0).is_err()); assert!(store::save(&root.0, &data).is_err()); assert_eq!(fs::read(root.file(store::STATE_FILE)).unwrap(), current);
}

#[test]
fn invalid_models_and_interrupted_writes_leave_existing_store_intact() {
    let root = TempRoot::new(); let mut data = demo::initial_data(); store::save(&root.0, &data).unwrap();
    let current = fs::read(root.file(store::STATE_FILE)).unwrap();
    data.settings.retention_count = 0; assert!(store::save(&root.0, &data).is_err());
    assert_eq!(fs::read(root.file(store::STATE_FILE)).unwrap(), current);
    data.settings.retention_count = 5;
    fs::write(root.file("state.json.tmp"), b"interrupted").unwrap();
    assert!(store::save(&root.0, &data).is_err());
    assert_eq!(fs::read(root.file(store::STATE_FILE)).unwrap(), current);
    assert_eq!(fs::read(root.file("state.json.tmp")).unwrap(), b"interrupted");
}

#[test]
fn failed_persistence_allows_runtime_to_keep_original_retention_and_backups() {
    let root = TempRoot::new(); let s = app(); store::save(&root.0, &s.data).unwrap();
    fs::write(root.file(store::STATE_FILE), b"broken").unwrap();
    let mut candidate = s.clone(); retention(&mut candidate, "1"); assert!(send(&mut candidate, "confirm:accept").persist);
    assert!(store::save(&root.0, &candidate.data).is_err());
    assert_eq!(s.data.settings.retention_count, 5); assert_eq!(s.data.backups.len(), 3);
}

#[cfg(unix)]
#[test]
fn data_symlinks_are_rejected_before_any_storage_write() {
    let root = TempRoot::new(); let outside = root.0.join("outside"); fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, root.0.join("data")).unwrap();
    assert!(store::load(&root.0).is_err()); assert!(store::save(&root.0, &demo::initial_data()).is_err());
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}
