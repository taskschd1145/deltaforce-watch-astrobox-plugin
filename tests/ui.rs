//! Structural acceptance of the host-native tree, independent of a browser/desktop.
use std::collections::BTreeSet;
use deltaforce_watch_astrobox_plugin::{
    contracts::{AppState, Confirmation, DamagePart, Page},
    demo, state::{format_time, money, outcome_label}, ui,
    view::Node,
};

const WIDTHS: [u32; 9] = [0, 360, 390, 639, 640, 768, 1023, 1024, 1440];
fn state() -> AppState { AppState::new(demo::initial_data()) }
fn walk(node: &Node) -> Vec<&Node> {
    fn visit<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
        out.push(node);
        for child in &node.children { visit(child, out); }
    }
    let mut out = Vec::new();
    visit(node, &mut out);
    out
}
fn all_text(node: &Node) -> String {
    walk(node).into_iter().filter_map(|node| node.text.as_deref()).collect::<Vec<_>>().join("\n")
}
fn events(node: &Node) -> BTreeSet<&str> { walk(node).into_iter().filter_map(|node| node.event_id.as_deref()).collect() }
fn event<'a>(node: &'a Node, id: &str) -> &'a Node {
    walk(node).into_iter().find(|node| node.event_id.as_deref() == Some(id)).unwrap_or_else(|| panic!("missing event {id}"))
}
fn prop<'a>(node: &'a Node, name: &str, value: &str) -> &'a Node {
    walk(node).into_iter().find(|node| node.props.get(name).map(String::as_str) == Some(value)).unwrap_or_else(|| panic!("missing prop {name}={value}"))
}
fn assert_native_tree(tree: &Node, width: u32) {
    let tags = ["div", "p", "span", "button", "input", "select", "option", "svg", "scroll-area", "progress", "checkbox", "switch"];
    let styles = ["background", "color", "font-size", "display", "flex-direction", "align-items", "justify-content", "width", "height", "min-width", "max-width", "min-height", "max-height", "padding", "padding-top", "padding-bottom", "padding-left", "padding-right", "margin", "margin-top", "margin-bottom", "margin-left", "margin-right", "border", "border-radius", "gap", "grid-template-columns", "flex-grow", "flex-shrink", "position", "top", "bottom", "left", "right", "z-index", "opacity"];
    let available = if width == 0 {360} else {width};
    assert_eq!(tree.styles.get("width").map(String::as_str), Some("100%"));
    for node in walk(tree) {
        assert!(tags.contains(&node.tag.as_str()), "unsupported tag {}", node.tag);
        assert!(!node.props.contains_key("class"));
        assert!(node.styles.get("margin").is_none_or(|value| !value.contains("auto")), "host margins are numeric");
        assert_ne!(node.styles.get("justify-content").map(String::as_str), Some("space-between"));
        assert_ne!(node.styles.get("align-items").map(String::as_str), Some("stretch"));
        let maximum = node.styles.get("max-width").map(String::as_str).unwrap_or("100%");
        // Pixel values are upper caps on fluid workspaces/modals, not fixed widths.
        assert!(["100%", "1280px", "560px"].contains(&maximum), "unexpected maximum width {maximum}");
        for key in node.styles.keys() { assert!(styles.contains(&key.as_str()), "unsupported style {key}"); }
        for key in ["width", "min-width"] {
            if let Some(pixels) = node.styles.get(key).and_then(|value| value.strip_suffix("px")).and_then(|value| value.parse::<u32>().ok()) {
                assert!(pixels <= available, "{key}={pixels}px exceeds {available}px");
            }
        }
        if node.event_id.is_some() {
            let height = node.styles.get("min-height").and_then(|value| value.strip_suffix("px")).and_then(|value| value.parse::<u32>().ok()).unwrap_or(0);
            assert!(height >= 44, "touch target {:?} is {height}px high", node.event_id);
            let minimum = node.styles.get("min-width").and_then(|value| value.strip_suffix("px")).and_then(|value| value.parse::<u32>().ok()).unwrap_or(0);
            assert!(minimum >= 44, "touch target {:?} is {minimum}px wide", node.event_id);
        }
        if let Some(grid) = node.styles.get("grid-template-columns") { assert!(grid.contains("minmax(0, 1fr)"), "grid tracks must shrink: {grid}"); }
        if let Some(radius) = node.styles.get("border-radius").and_then(|value| value.strip_suffix("px")).and_then(|value| value.parse::<u32>().ok()) { assert!(radius <= 4); }
    }
}

#[test]
fn renders_every_breakpoint_and_four_pages_with_native_styles() {
    for width in WIDTHS {
        for page in [Page::Matches, Page::MatchDetail, Page::Services, Page::Settings] {
            let mut state = state();
            state.page = page;
            state.selected_match_id = Some(state.data.snapshot.matches[0].id.clone());
            let tree = ui::render(&state, width, if width == 0 {0} else {900});
            assert_native_tree(&tree, width);
            let expected_layout = if width >= 1024 {"desktop"} else if width >= 640 {"tablet"} else {"phone"};
            assert_eq!(tree.props.get("data-layout").map(String::as_str), Some(expected_layout));
            let labels = all_text(&tree);
            assert!(labels.contains("三角洲行动同步器"));
            assert!(labels.contains("UI DEMO · 演示数据"));
            assert!(labels.contains("未连接手表，数据未更新"));
            assert!(labels.contains(&format_time(Some(state.data.snapshot.captured_at), state.timezone_offset_minutes)));
            assert!(labels.contains(&format!("来源 · {}", state.data.snapshot.source)));
            let ids = events(&tree);
            for id in ["nav:matches", "nav:services", "nav:settings"] { assert!(ids.contains(id), "{width} {page:?}: missing {id}"); }
            prop(&tree, "data-navigation", if width >= 1024 {"sidebar"} else if width >= 640 {"compact"} else {"bottom"});
            if page == Page::MatchDetail { event(&tree, "back:matches"); }
        }
    }
}

#[test]
fn phone_is_single_column_and_tablet_uses_two_content_columns() {
    for width in [0, 360, 390, 639, 640, 768, 1023, 1024, 1440] {
        let tree = ui::render(&state(), width, 800);
        let list = prop(&tree, "aria-label", "对局列表");
        assert_eq!(list.styles.get("grid-template-columns").unwrap(), if width < 640 {"repeat(1, minmax(0, 1fr))"} else {"repeat(2, minmax(0, 1fr))"});
        let has_context = walk(&tree).iter().any(|node| node.props.get("aria-label").map(String::as_str) == Some("快照上下文"));
        assert_eq!(has_context, width >= 1024);
    }
}

#[test]
fn desktop_overview_has_four_compact_stat_columns_and_smaller_layouts_keep_two() {
    for width in WIDTHS {
        let tree = ui::render(&state(), width, 840);
        let stats = prop(&tree, "aria-label", "战绩统计");
        assert_eq!(stats.children.len(), 4);
        assert_eq!(stats.styles.get("grid-template-columns").map(String::as_str), Some(if width >= 1024 {"repeat(4, minmax(0, 1fr))"} else {"repeat(2, minmax(0, 1fr))"}));
        for label in ["档案对局", "成功 / 胜利", "累计击败", "烽火带出货值"] { assert!(all_text(stats).contains(label)); }
    }
}

#[test]
fn compact_chrome_preserves_status_metadata_and_reserves_feedback_for_real_notices() {
    let mut state = state();
    let original_notice = state.notice.clone();
    for width in WIDTHS {
        let tree = ui::render(&state, width, 840);
        let header = prop(&tree, "aria-label", "应用标题");
        assert_eq!(header.styles.get("padding").map(String::as_str), Some(if width < 640 {"10px 12px"} else {"12px 24px"}));
        let summary = prop(&tree, "data-connection-summary", "true");
        assert_eq!(summary.children.len(), 2, "status plus a combined timestamp/source line");
        assert_eq!(summary.styles.get("flex-direction").map(String::as_str), Some(if width < 640 {"column"} else {"row"}));
        assert!(all_text(&summary.children[0]).contains("未连接手表，数据未更新"));
        let metadata = all_text(&summary.children[1]);
        assert!(metadata.contains(&format_time(Some(state.data.snapshot.captured_at), state.timezone_offset_minutes)));
        assert!(metadata.contains(&format!("来源 · {}", state.data.snapshot.source)));
        assert!(all_text(&tree).contains("UI DEMO · 演示数据"));
        assert!(!walk(&tree).iter().any(|node| node.props.get("aria-label").map(String::as_str) == Some("操作反馈")));
    }
    assert_eq!(state.notice, original_notice, "compact launch chrome is presentation-only");
    state.notice = "已创建当前模拟快照备份：fixture-backup。".into();
    let tree = ui::render(&state, 1440, 840);
    assert!(all_text(prop(&tree, "aria-label", "操作反馈")).contains(&state.notice));
}

#[test]
fn known_render_height_bounds_native_scroll_area_and_keeps_chrome_outside() {
    let mut state = state();
    for width in WIDTHS {
        let tree = ui::render(&state, width, 840);
        let expected = if width < 640 {840 - 216} else {840 - 140};
        let pixels = format!("{expected}px");
        let body = prop(&tree, "aria-label", "主工作区");
        let scroll = prop(&tree, "aria-label", "战绩内容");
        for node in [body, scroll] {
            assert_eq!(node.styles.get("height"), Some(&pixels));
            assert_eq!(node.styles.get("max-height"), Some(&pixels));
        }
        assert_eq!(body.styles.get("flex-grow").map(String::as_str), Some("0"));
        assert_eq!(body.styles.get("flex-shrink").map(String::as_str), Some("0"));
        assert!(!walk(scroll).iter().any(|node| matches!(node.props.get("aria-label").map(String::as_str), Some("应用标题" | "连接与快照状态" | "底部主导航"))));
        let unknown = ui::render(&state, width, 0);
        assert!(!prop(&unknown, "aria-label", "战绩内容").styles.contains_key("height"));
        assert!(!prop(&unknown, "aria-label", "主工作区").styles.contains_key("height"));
    }
    state.viewed_backup_id = Some(state.data.backups[0].id.clone());
    let preview = ui::render(&state, 1440, 840);
    assert_eq!(prop(&preview, "aria-label", "战绩内容").styles.get("height").map(String::as_str), Some("612px"));
    let tiny = ui::render(&state, 390, 100);
    assert_eq!(prop(&tiny, "aria-label", "战绩内容").styles.get("height").map(String::as_str), Some("0px"));
}

#[test]
fn phone_primary_navigation_is_pinned_above_overflow_and_unknown_height_stays_flowing() {
    for width in [0, 360, 390, 420, 639] {
        for page in [Page::Matches, Page::MatchDetail, Page::Services, Page::Settings] {
            let mut state = state();
            state.page = page;
            state.selected_match_id = Some(state.data.snapshot.matches[0].id.clone());
            let tree = ui::render(&state, width, 840);
            let nav = prop(&tree, "data-navigation", "bottom");
            assert!(tree.children.iter().any(|node| node.props.get("data-navigation").map(String::as_str) == Some("bottom")), "navigation is anchored directly to the viewport root");
            for (key, value) in [("position", "absolute"), ("left", "0"), ("right", "0"), ("bottom", "0"), ("height", "64px"), ("max-height", "64px"), ("z-index", "50")] {
                assert_eq!(nav.styles.get(key).map(String::as_str), Some(value));
            }
            for id in ["nav:matches", "nav:services", "nav:settings"] { event(nav, id); }
            assert_eq!(prop(&tree, "data-content-layout", "true").styles.get("padding-bottom").map(String::as_str), Some("64px"));
            let unknown = ui::render(&state, width, 0);
            let flow_nav = prop(&unknown, "data-navigation", "bottom");
            assert!(!flow_nav.styles.contains_key("position"));
            assert!(!flow_nav.styles.contains_key("bottom"));
            assert!(!prop(&unknown, "data-content-layout", "true").styles.contains_key("padding-bottom"));
        }
    }
}

#[test]
fn twelve_snapshot_records_render_three_information_lines_without_fabrication() {
    let state = state();
    assert!(state.data.snapshot.matches.len() >= 12);
    let tree = ui::render(&state, 390, 844);
    let list = prop(&tree, "aria-label", "对局列表");
    assert_eq!(list.children.len(), state.data.snapshot.matches.len());
    for record in &state.data.snapshot.matches {
        let card = event(list, &format!("match:{}", record.id));
        assert_eq!(card.children.len(), 3);
        let first = prop(card, "data-line", "result-value-kills");
        let first_text = all_text(first);
        assert!(first_text.contains(outcome_label(record)));
        assert!(first_text.contains(&format!("{} 击败", record.kills)));
        assert!(first_text.contains(&if record.mode == "battlefield" {format!("{} 分", record.score)} else {format!("{} 货值", money(record.loot_value))}));
        let second = all_text(prop(card, "data-line", "mode-map-difficulty"));
        assert!(second.contains(&record.map_name));
        assert!(second.contains(&record.difficulty));
        assert!(all_text(prop(card, "data-line", "started-at")).contains(&format_time(record.started_at, state.timezone_offset_minutes)));
    }
}

#[test]
fn mode_filters_and_stats_follow_current_snapshot() {
    let mut state = state();
    for filter in ["all", "operations", "battlefield"] {
        state.history_filter = filter.into();
        let tree = ui::render(&state, 768, 900);
        for id in ["filter:all", "filter:operations", "filter:battlefield"] { event(&tree, id); }
        let active = event(&tree, &format!("filter:{filter}"));
        assert_eq!(active.props.get("aria-pressed").map(String::as_str), Some("true"));
        let expected = state.data.snapshot.matches.iter().filter(|record| filter == "all" || record.mode == filter).count();
        assert_eq!(prop(&tree, "aria-label", "对局列表").children.len(), expected);
        prop(&tree, "aria-label", "战绩统计");
    }
    state.data.snapshot.matches.clear();
    let tree = ui::render(&state, 360, 700);
    assert!(all_text(&tree).contains("暂无对局"));
}

#[test]
fn all_five_body_parts_include_abdomen_and_separate_damage_from_injury() {
    let mut state = state();
    state.page = Page::MatchDetail;
    let record = &mut state.data.snapshot.matches[0];
    record.damage = [("head", 313, 7), ("chest", 471, 18), ("abdomen", 529, 29), ("arm", 587, 31), ("leg", 643, 43)].into_iter().map(|(part, damage, injury)| DamagePart {part: part.into(), damage_taken: Some(damage), injury_at_end: Some(injury)}).collect();
    state.selected_match_id = Some(record.id.clone());
    for width in WIDTHS {
        let tree = ui::render(&state, width, 800);
        let details = prop(&tree, "aria-label", "五部位伤害详情");
        for (part, damage, injury) in [("head", "313", "7"), ("chest", "471", "18"), ("abdomen", "529", "29"), ("arm", "587", "31"), ("leg", "643", "43")] {
            prop(details, "data-body-part", part);
            let detail = prop(details, "data-damage-part", part);
            assert_eq!(prop(detail, "data-measure", "damage-taken").text.as_deref(), Some(damage));
            assert_eq!(prop(detail, "data-measure", "injury-at-end").text.as_deref(), Some(injury));
        }
        assert!(all_text(details).contains("腹部"));
        assert!(all_text(details).contains("累计承伤"));
        assert!(all_text(details).contains("结束伤势"));
    }
}

#[test]
fn plural_body_aliases_and_missing_damage_remain_readable() {
    let mut state = state();
    state.page = Page::MatchDetail;
    let record = &mut state.data.snapshot.matches[0];
    state.selected_match_id = Some(record.id.clone());
    record.damage = vec![DamagePart {part: "arms".into(), damage_taken: Some(88), injury_at_end: None}, DamagePart {part: "legs".into(), damage_taken: None, injury_at_end: Some(21)}];
    let tree = ui::render(&state, 360, 800);
    assert_eq!(prop(prop(&tree, "data-damage-part", "arm"), "data-measure", "damage-taken").text.as_deref(), Some("88"));
    assert_eq!(prop(prop(&tree, "data-damage-part", "leg"), "data-measure", "injury-at-end").text.as_deref(), Some("21"));
    assert_eq!(prop(prop(&tree, "data-damage-part", "abdomen"), "data-measure", "damage-taken").text.as_deref(), Some("—"));
}

#[test]
fn failed_operation_still_exposes_safe_container_loot_and_exact_values() {
    let mut state = state();
    state.page = Page::MatchDetail;
    let record = state.data.snapshot.matches.iter().find(|record| record.mode == "operations" && record.outcome == "killed").unwrap();
    let loot = record.loot.clone();
    assert!(loot.iter().any(|item| item.compartment == "safe"));
    state.selected_match_id = Some(record.id.clone());
    let tree = ui::render(&state, 390, 844);
    let detail = prop(&tree, "aria-label", "战利品明细");
    assert!(all_text(detail).contains("安全箱"));
    for item in &loot {
        let card = event(detail, &format!("item:{}", item.id));
        assert!(all_text(card).contains(&item.name));
        assert!(all_text(card).contains(&format!("×{}", item.quantity)));
        assert!(all_text(card).contains(&money(item.unit_value.saturating_mul(u64::from(item.quantity)))));
    }
}

#[test]
fn battlefield_detail_has_score_kills_deaths_captures_revives_xp() {
    let mut state = state();
    state.page = Page::MatchDetail;
    let record = state.data.snapshot.matches.iter().find(|record| record.mode == "battlefield").unwrap();
    state.selected_match_id = Some(record.id.clone());
    let tree = ui::render(&state, 1024, 900);
    let stats = all_text(prop(&tree, "aria-label", "全面战场统计"));
    assert!(stats.contains(&record.score.to_string()));
    assert!(stats.contains(&format!("{} / {}", record.kills, record.deaths)));
    assert!(stats.contains(&format!("{} / {}", record.captures, record.revives)));
    assert!(stats.contains(&format!("+{} XP", record.xp_gain)));
}

#[test]
fn all_services_tabs_render_at_every_breakpoint_with_player_data() {
    for width in WIDTHS {
        for tab in ["inventory", "facilities", "missions"] {
            let mut state = state();
            state.page = Page::Services;
            state.services_tab = tab.into();
            let tree = ui::render(&state, width, 900);
            assert_native_tree(&tree, width);
            prop(&tree, "data-services-tab", tab);
            for id in ["services:inventory", "services:facilities", "services:missions"] {event(&tree, id);}
            let labels = all_text(&tree);
            assert!(labels.contains(&state.data.snapshot.player.name));
            assert!(labels.contains(&money(state.data.snapshot.player.funds)));
            assert!(labels.contains(&format!("Lv.{}", state.data.snapshot.player.level)));
            assert!(labels.contains(&format!("{} / {}", state.data.snapshot.player.warehouse_used, state.data.snapshot.player.warehouse_capacity)));
            match tab {
                "facilities" => {
                    for facility in &state.data.snapshot.facilities { assert!(labels.contains(&facility.name)); assert!(labels.contains(&facility.effect)); }
                    assert!(labels.contains("仓库"));
                    assert!(labels.contains("训练"));
                }
                "missions" => { assert!(labels.contains("规划演示")); for id in ["mission:all", "mission:pending", "mission:completed"] {event(&tree, id);} }
                _ => {event(&tree, "query:inventory");}
            }
        }
    }
}

#[test]
fn inventory_query_quality_category_and_long_names_flow_from_state() {
    let mut state = state();
    state.page = Page::Services;
    let long_name = "超长物品名称用于验证窄屏自动换行与完整文字展示".repeat(8);
    let item = &mut state.data.snapshot.inventory[0];
    item.name = long_name.clone();
    let item_id = item.id.clone();
    state.inventory_query = "超长物品".into();
    state.inventory_quality = item.quality.clone();
    state.inventory_category = item.category.clone();
    let tree = ui::render(&state, 360, 780);
    assert_native_tree(&tree, 360);
    assert_eq!(event(&tree, "query:inventory").props.get("value"), Some(&state.inventory_query));
    let items = prop(&tree, "aria-label", "库存物品网格");
    assert_eq!(items.children.len(), 1);
    assert!(all_text(event(items, &format!("item:{item_id}"))).contains(&long_name));
    for quality in ["all", "白", "绿", "蓝", "紫", "金", "红"] {event(&tree, &format!("quality:{quality}"));}
    for category in ["all", "装备", "弹药", "药品", "贵重品", "房卡"] {event(&tree, &format!("category:{category}"));}
    state.inventory_query = "missing-query".into();
    assert!(all_text(&ui::render(&state, 360, 780)).contains("没有匹配的物品"));
}

#[test]
fn inventory_filters_match_each_contract_quality_and_category() {
    let mut state = state();
    state.page = Page::Services;
    for quality in ["all", "白", "绿", "蓝", "紫", "金", "红"] {
        for category in ["all", "装备", "弹药", "药品", "贵重品", "房卡"] {
            state.inventory_quality = quality.into();
            state.inventory_category = category.into();
            let expected = state.data.snapshot.inventory.iter().filter(|item| (quality == "all" || item.quality == quality) && (category == "all" || item.category == category)).count();
            let tree = ui::render(&state, 640, 900);
            assert_eq!(prop(&tree, "aria-label", "库存物品网格").children.len(), expected);
        }
    }
}

#[test]
fn mission_pending_completed_filters_follow_progress_and_target() {
    let mut state = state();
    state.page = Page::Services;
    state.services_tab = "missions".into();
    for filter in ["all", "pending", "completed"] {
        state.mission_filter = filter.into();
        let tree = ui::render(&state, 390, 900);
        let actual: BTreeSet<_> = walk(&tree).into_iter().filter_map(|node| node.props.get("data-mission-id").cloned()).collect();
        let expected: BTreeSet<_> = state.data.snapshot.missions.iter().filter(|mission| match filter {"pending" => mission.progress < mission.target, "completed" => mission.progress >= mission.target, _ => true}).map(|mission| mission.id.clone()).collect();
        assert_eq!(actual, expected);
        assert!(all_text(&tree).contains("规划演示"));
    }
}

#[test]
fn settings_lists_every_backup_and_all_four_actions_at_all_sizes() {
    let mut state = state();
    state.page = Page::Settings;
    state.retention_input = "7".into();
    for width in WIDTHS {
        let tree = ui::render(&state, width, 900);
        for id in ["device:toggle", "backup:create", "settings:auto-backup", "settings:retention-input", "settings:retention-apply"] {event(&tree, id);}
        assert_eq!(event(&tree, "settings:retention-input").props.get("value").map(String::as_str), Some("7"));
        for backup in &state.data.backups {
            let card = prop(&tree, "data-backup-id", &backup.id);
            assert!(all_text(card).contains(&backup.device_name));
            assert!(all_text(card).contains(&format_time(Some(backup.created_at), state.timezone_offset_minutes)));
            for action in ["view", "restore", "export", "delete"] {event(card, &format!("backup:{action}:{}", backup.id));}
        }
    }
}

#[test]
fn simulated_connection_and_auto_backup_switch_values_are_bound() {
    let mut state = state();
    state.page = Page::Settings;
    for selected in [false, true] {
        state.connected = selected;
        state.data.settings.auto_backup_on_launch = selected;
        let tree = ui::render(&state, 390, 844);
        assert!(all_text(&tree).contains(if selected {"演示连接 ·"} else {"未连接手表，数据未更新"}));
        for id in ["device:toggle", "settings:auto-backup"] {
            let toggle = event(&tree, id);
            assert_eq!(toggle.tag, "button", "toggles must use the host's uniform Click path");
            assert_eq!(toggle.props.get("role").map(String::as_str), Some("switch"));
            for key in ["selected", "value", "aria-checked"] {
                assert_eq!(toggle.props.get(key).map(String::as_str), Some(if selected {"true"} else {"false"}));
            }
            assert!(all_text(toggle).contains(if selected {"已开启"} else {"已关闭"}));
        }
    }
}

#[test]
fn confirmation_modals_cover_root_and_expose_only_safe_confirmation_actions() {
    let mut state = state();
    state.page = Page::Settings;
    let id = state.data.backups[0].id.clone();
    state.viewed_backup_id = Some(id.clone());
    for confirmation in [Confirmation::Restore(id.clone()), Confirmation::DeleteBackup(id.clone()), Confirmation::SetRetention(2)] {
        state.confirmation = Some(confirmation.clone());
        for width in WIDTHS {
            let tree = ui::render(&state, width, 850);
            assert_native_tree(&tree, width);
            let modal = prop(&tree, "aria-modal", "true");
            assert_eq!(modal.styles.get("position").map(String::as_str), Some("absolute"));
            for edge in ["top", "bottom", "left", "right"] { assert_eq!(modal.styles.get(edge).map(String::as_str), Some("0")); }
            assert_eq!(events(&tree), BTreeSet::from(["confirm:accept", "confirm:cancel"]));
            assert!(all_text(modal).contains("本插件的本地快照与备份库"));
            match confirmation {Confirmation::SetRetention(_) => assert!(all_text(modal).contains("最近 2 份")), _ => assert!(all_text(modal).contains(&id))}
        }
    }
}

#[test]
fn item_modal_uses_inventory_or_loot_contract_and_exposes_close_only() {
    let mut state = state();
    let inventory_item = state.data.snapshot.inventory[0].clone();
    let loot_item = state.data.snapshot.matches.iter().flat_map(|record| record.loot.iter()).next().unwrap().clone();
    for item in [inventory_item, loot_item] {
        state.selected_item_id = Some(item.id.clone());
        for width in WIDTHS {
            let tree = ui::render(&state, width, 900);
            assert_native_tree(&tree, width);
            let modal = prop(&tree, "aria-label", "物品详情");
            let labels = all_text(modal);
            assert!(labels.contains(&item.name));
            assert!(labels.contains(&item.quantity.to_string()));
            assert!(labels.contains(&money(item.unit_value)));
            assert!(labels.contains(&money(item.unit_value.saturating_mul(u64::from(item.quantity)))));
            assert!(labels.contains("耐久"));
            assert_eq!(events(&tree), BTreeSet::from(["item:close"]));
        }
    }
}

#[test]
fn backup_preview_uses_display_snapshot_for_banner_all_business_pages_and_item_lookup() {
    let mut state = state();
    let backup = &mut state.data.backups[0];
    backup.snapshot.source = "preview-source-only".into();
    backup.snapshot.player.name = "预览专属干员".into();
    backup.snapshot.player.funds = 123_456_789;
    backup.snapshot.matches[0].map_name = "预览专属地图".into();
    backup.snapshot.inventory[0].name = "预览专属物品".into();
    let backup_id = backup.id.clone();
    let match_id = backup.snapshot.matches[0].id.clone();
    let item_id = backup.snapshot.inventory[0].id.clone();
    let active_before = state.data.snapshot.clone();
    state.viewed_backup_id = Some(backup_id);
    for page in [Page::Matches, Page::MatchDetail, Page::Services, Page::Settings] {
        state.page = page;
        state.selected_match_id = Some(match_id.clone());
        let tree = ui::render(&state, 1440, 1000);
        let labels = all_text(&tree);
        assert!(labels.contains("正在查看备份快照"));
        assert!(labels.contains("来源 · preview-source-only"));
        event(&tree, "preview:exit");
        assert!(labels.contains("预览专属干员"));
        assert!(labels.contains(&money(123_456_789)));
        if page == Page::Matches || page == Page::MatchDetail { assert!(labels.contains("预览专属地图")); }
        if page == Page::Services { assert!(labels.contains("预览专属物品")); }
    }
    state.selected_item_id = Some(item_id);
    assert!(all_text(&ui::render(&state, 390, 844)).contains("预览专属物品"));
    assert_eq!(state.data.snapshot, active_before);
}

#[test]
fn empty_snapshot_missing_selection_and_large_values_are_total_and_pure() {
    let mut state = state();
    state.data.snapshot.matches.clear();
    state.data.snapshot.inventory.clear();
    state.data.snapshot.missions.clear();
    state.data.snapshot.facilities.clear();
    state.data.backups.clear();
    state.data.snapshot.player.funds = u64::MAX;
    state.selected_match_id = Some("stale-match".into());
    state.selected_item_id = Some("stale-item".into());
    state.notice = "本地操作已完成".into();
    let original = state.data.clone();
    for page in [Page::Matches, Page::MatchDetail, Page::Services, Page::Settings] {
        state.page = page;
        for width in WIDTHS {
            let first = ui::render(&state, width, 0);
            let second = ui::render(&state, width, 0);
            assert_native_tree(&first, width);
            assert_eq!(serde_json::to_value(first).unwrap(), serde_json::to_value(second).unwrap());
        }
    }
    assert_eq!(state.data, original);
}

#[test]
fn item_total_value_saturates_without_overflow() {
    let mut state = state();
    state.page = Page::Services;
    let item = &mut state.data.snapshot.inventory[0];
    item.quantity = u32::MAX;
    item.unit_value = u64::MAX;
    let id = item.id.clone();
    let tree = ui::render(&state, 360, 800);
    assert!(all_text(event(&tree, &format!("item:{id}"))).contains(&money(u64::MAX)));
}

#[test]
fn snapshot_source_notice_timezone_and_busy_feedback_remain_visible() {
    let mut state = state();
    state.data.snapshot.source = "fixture-local-snapshot".into();
    state.timezone_offset_minutes = -300;
    state.notice = "备份导出已准备就绪".into();
    state.busy = true;
    let tree = ui::render(&state, 360, 800);
    let labels = all_text(&tree);
    assert!(labels.contains("来源 · fixture-local-snapshot"));
    assert!(labels.contains(&format_time(Some(state.data.snapshot.captured_at), -300)));
    assert!(labels.contains(&state.notice));
    assert!(labels.contains("正在处理本地操作"));
}

#[test]
fn unknown_render_size_uses_fluid_phone_canvas_and_finite_metadata() {
    let state = state();
    let tree = ui::render(&state, 0, 0);
    assert_eq!(tree.props.get("data-layout").map(String::as_str), Some("phone"));
    assert_eq!(tree.styles.get("width").map(String::as_str), Some("100%"));
    assert_eq!(tree.styles.get("height").map(String::as_str), Some("100%"));
    prop(&tree, "data-navigation", "bottom");
    let large = ui::render(&state, u32::MAX, u32::MAX);
    assert_eq!(large.props.get("data-layout").map(String::as_str), Some("desktop"));
    assert_eq!(large.styles.get("height").map(String::as_str), Some("4294967295px"));
}

#[test]
fn detail_keeps_matches_navigation_active_and_confirmation_has_priority_over_item() {
    let mut state = state();
    state.page = Page::MatchDetail;
    state.selected_match_id = Some(state.data.snapshot.matches[0].id.clone());
    for width in [390, 768, 1440] {
        let tree = ui::render(&state, width, 900);
        assert_eq!(event(&tree, "nav:matches").props.get("aria-pressed").map(String::as_str), Some("true"));
        assert_eq!(event(&tree, "nav:services").props.get("aria-pressed").map(String::as_str), Some("false"));
        event(&tree, "back:matches");
    }
    state.selected_item_id = Some(state.data.snapshot.inventory[0].id.clone());
    state.confirmation = Some(Confirmation::DeleteBackup(state.data.backups[0].id.clone()));
    let tree = ui::render(&state, 390, 844);
    assert_eq!(events(&tree), BTreeSet::from(["confirm:accept", "confirm:cancel"]));
    prop(&tree, "aria-label", "本地备份操作确认");
}

#[test]
fn tactical_palette_preserves_requested_quality_colors() {
    assert_eq!(ui::palette::BACKGROUND, "#171b1a");
    assert_eq!(ui::palette::TEXT, "#f4f4f0");
    assert_eq!(ui::palette::SECONDARY, "#b9c7bd");
    assert_eq!(ui::palette::GREEN, "#326d52");
    assert_eq!(ui::palette::AMBER, "#f2bd74");
    for (quality, color) in [("白", "#48514d"), ("绿", "#286a49"), ("蓝", "#315f9f"), ("紫", "#694295"), ("金", "#805c16"), ("红", "#9c3440")] {assert_eq!(ui::palette::quality(quality), color);}
}
