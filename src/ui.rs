//! Pure, host-native UI rendering. All editable data and routing belong to AppState.
use crate::contracts::{AppState, Confirmation, DamagePart, Item, MatchRecord, Page};
use crate::state::{format_time, money, outcome_label};
use crate::view::{Layout, Node};

/// Deliberately small tactical palette, shared by badges, cards and the body diagram.
pub mod palette {
    pub const BACKGROUND: &str = "#171b1a";
    pub const PANEL: &str = "#202723";
    pub const INSET: &str = "#121815";
    pub const LINE: &str = "#39463e";
    pub const TEXT: &str = "#f4f4f0";
    pub const SECONDARY: &str = "#b9c7bd";
    pub const GREEN: &str = "#326d52";
    pub const AMBER: &str = "#f2bd74";
    pub const RED: &str = "#9c3440";
    pub fn quality(quality: &str) -> &'static str {
        match quality {
            "绿" => "#286a49",
            "蓝" => "#315f9f",
            "紫" => "#694295",
            "金" => "#805c16",
            "红" => "#9c3440",
            _ => "#48514d",
        }
    }
}
use palette as p;

fn el(tag: &str) -> Node {
    Node::new(tag)
        .style("min-width", "0")
        .style("max-width", "100%")
        .style("flex-shrink", "1")
}
fn col(gap: u32) -> Node {
    el("div")
        .style("display", "flex")
        .style("flex-direction", "column")
        .style("gap", format!("{gap}px"))
}
fn row(gap: u32) -> Node {
    el("div")
        .style("display", "flex")
        .style("flex-direction", "row")
        .style("align-items", "center")
        .style("gap", format!("{gap}px"))
}
fn grid(columns: usize, gap: u32) -> Node {
    el("div")
        .style("display", "grid")
        .style(
            "grid-template-columns",
            format!("repeat({columns}, minmax(0, 1fr))"),
        )
        .style("gap", format!("{gap}px"))
}
fn text(value: impl Into<String>) -> Node {
    el("p").text(value).style("margin", "0")
}
fn muted(value: impl Into<String>) -> Node {
    text(value)
        .style("color", p::SECONDARY)
        .style("font-size", "12px")
}
fn heading(value: impl Into<String>) -> Node {
    text(value).style("font-size", "22px")
}
fn panel() -> Node {
    col(14)
        .style("background", p::PANEL)
        .style("border", format!("1px solid {}", p::LINE))
        .style("border-radius", "4px")
        .style("padding", "16px")
}
fn badge(value: impl Into<String>, color: &str) -> Node {
    el("span")
        .text(value)
        .style("background", color)
        .style("color", p::TEXT)
        .style("padding", "4px 8px")
        .style("border-radius", "3px")
        .style("font-size", "12px")
}
fn button(label: impl Into<String>, event: impl Into<String>, active: bool) -> Node {
    el("button")
        .text(label)
        .event(event)
        .style("display", "flex")
        .style("align-items", "center")
        .style("justify-content", "center")
        .style("min-height", "44px")
        .style("min-width", "44px")
        .style("padding", "8px 12px")
        .style(
            "border",
            format!("1px solid {}", if active { p::GREEN } else { p::LINE }),
        )
        .style("background", if active { p::GREEN } else { p::INSET })
        .style("color", p::TEXT)
        .style("border-radius", "3px")
        .style("font-size", "14px")
        .prop("aria-pressed", active)
}
fn section_title(kicker: &str, title: &str, detail: impl Into<String>) -> Node {
    col(6)
        .child(muted(kicker).style("color", p::AMBER))
        .child(heading(title))
        .child(muted(detail))
}
fn metric(label: &str, value: impl Into<String>, accent: bool) -> Node {
    col(8)
        .style("background", p::INSET)
        .style("padding", "12px")
        .style("border-radius", "3px")
        .child(muted(label))
        .child(
            text(value)
                .style("font-size", "22px")
                .style("color", if accent { p::AMBER } else { p::TEXT }),
        )
}
fn columns(layout: Layout) -> usize {
    if layout == Layout::Phone { 1 } else { 2 }
}
fn source_label(source: &str) -> Node {
    muted(format!("来源 · {source}"))
}
fn time(state: &AppState, timestamp: Option<i64>) -> String {
    format_time(timestamp, state.timezone_offset_minutes)
}
fn total_value(item: &Item) -> u64 {
    item.unit_value.saturating_mul(u64::from(item.quantity))
}
fn compartment_label(value: &str) -> &str {
    match value {
        "safe" | "secure" | "safe-container" => "安全箱",
        "backpack" => "背包",
        "warehouse" => "仓库",
        _ => value,
    }
}
fn mode_label(record: &MatchRecord) -> &str {
    match record.mode.as_str() {
        "operations" => "烽火地带",
        "battlefield" => "全面战场",
        _ => &record.mode,
    }
}
fn success(record: &MatchRecord) -> bool {
    matches!(record.outcome.as_str(), "extracted" | "victory")
}
fn page_label(page: Page) -> &'static str {
    match page {
        Page::Matches => "战绩",
        Page::MatchDetail => "对局详情",
        Page::Services => "特勤处",
        Page::Settings => "设置",
    }
}

/// Render a deterministic UI tree. Unknown render dimensions use the phone breakpoint.
/// No IO, device calls, mutation, assets, browser CSS or random values enter this function.
pub fn render(state: &AppState, width: u32, height: u32) -> Node {
    let layout = Layout::for_width(width);
    let layout_id = match layout {
        Layout::Phone => "phone",
        Layout::Tablet => "tablet",
        Layout::Desktop => "desktop",
    };
    let mut root = col(0)
        .style("position", "relative")
        .style("background", p::BACKGROUND)
        .style("color", p::TEXT)
        .style("width", "100%")
        .style(
            "height",
            if height == 0 {
                "100%".into()
            } else {
                format!("{height}px")
            },
        )
        .style("font-size", "14px")
        .prop("data-layout", layout_id)
        .prop("aria-label", layout.label())
        .prop("data-page", page_label(state.page));
    root = root
        .child(header(layout))
        .child(connection_bar(state, layout));
    // Modal rendering exposes only its own actions, including to accessibility/event traversal.
    if let Some(confirmation) = &state.confirmation {
        return root.child(confirmation_modal(state, confirmation, layout));
    }
    if let Some(item) = state.selected_item() {
        return root.child(item_modal(item, layout));
    }
    let content = match state.page {
        Page::Matches => matches_page(state, layout),
        Page::MatchDetail => match_detail(state, layout),
        Page::Services => services_page(state, layout),
        Page::Settings => settings_page(state, layout),
    };
    // Native ScrollArea needs an explicit viewport, not an intrinsic content height.
    // These budgets include our own chrome only; height is already the host's render area.
    let chrome_budget = if layout == Layout::Phone {
        88 + 64 + 64
    } else {
        96 + 44
    };
    let preview_budget = if state.viewed_backup_id.is_some() {
        88
    } else {
        0
    };
    let scroll_height = (height > 0).then(|| height.saturating_sub(chrome_budget + preview_budget));
    let mut main = el("scroll-area")
        .style("flex-grow", "1")
        .style("min-height", "0")
        .prop("aria-label", format!("{}内容", page_label(state.page)));
    if let Some(pixels) = scroll_height {
        main = main
            .style("height", format!("{pixels}px"))
            .style("max-height", format!("{pixels}px"));
    }
    let mut workspace = row(20)
        .style("align-items", "flex-start")
        .style(
            "padding",
            if layout == Layout::Phone {
                "16px 12px"
            } else {
                "24px"
            },
        )
        .style("width", "100%")
        .style("max-width", "1280px")
        .prop("data-content-layout", "true");
    if layout == Layout::Phone && scroll_height.is_some() {
        workspace = workspace.style("padding-bottom", "64px");
    }
    let mut page = col(18).style("flex-grow", "1");
    // The permanent DEMO subtitle already communicates the launch hint; keep room for records.
    let is_launch_hint =
        state.notice == "UI Demo · 所有界面数据均为模拟，点击模拟连接读取示例快照。";
    if !state.notice.is_empty() && !is_launch_hint {
        page = page.child(
            text(&state.notice)
                .style("padding", "12px")
                .style("background", p::INSET)
                .style("border", format!("1px solid {}", p::LINE))
                .prop("role", "status")
                .prop("aria-label", "操作反馈"),
        );
    }
    if state.busy {
        page = page.child(muted("正在处理本地操作…").prop("role", "status"));
    }
    workspace = workspace.child(page.child(content));
    if layout == Layout::Desktop {
        workspace = workspace.child(context_panel(state));
    }
    main = main.child(
        row(0)
            .style("align-items", "flex-start")
            .style("justify-content", "center")
            .child(workspace),
    );
    // An omitted alignment preserves native flex stretch; Host has no stretch setter.
    let mut body = el("div")
        .style("display", "flex")
        .style("flex-direction", "row")
        .style("flex-grow", "1")
        .style("min-height", "0")
        .prop("aria-label", "主工作区");
    if let Some(pixels) = scroll_height {
        body = body
            .style("height", format!("{pixels}px"))
            .style("max-height", format!("{pixels}px"))
            .style("flex-grow", "0")
            .style("flex-shrink", "0");
    }
    if layout != Layout::Phone {
        body = body.child(navigation(state, layout));
    }
    root = root.child(body.child(main));
    if layout == Layout::Phone {
        let mut nav = navigation(state, layout);
        if scroll_height.is_some() {
            // Pin the primary actions to the render viewport even when Host content overflows.
            nav = nav
                .style("position", "absolute")
                .style("left", "0")
                .style("right", "0")
                .style("bottom", "0")
                .style("height", "64px")
                .style("max-height", "64px")
                .style("z-index", "50");
        }
        root = root.child(nav);
    }
    root
}

fn header(layout: Layout) -> Node {
    let compact = layout == Layout::Phone;
    row(12)
        .style("padding", if compact { "10px 12px" } else { "12px 24px" })
        .style("flex-shrink", "0")
        .style("border", format!("1px solid {}", p::LINE))
        .prop("aria-label", "应用标题")
        .child(
            el("div")
                .text("Δ")
                .style("font-size", "28px")
                .style("color", p::AMBER)
                .style("width", "38px")
                .style("flex-shrink", "0")
                .prop("aria-hidden", "true"),
        )
        .child(
            col(5)
                .style("flex-grow", "1")
                .child(
                    text("三角洲行动同步器")
                        .style("font-size", if compact { "20px" } else { "24px" }),
                )
                .child(muted("UI DEMO · 演示数据")),
        )
        .child(badge("0.1.0", p::INSET))
}
fn connection_bar(state: &AppState, layout: Layout) -> Node {
    let label = if state.connected {
        "演示连接 · 模拟手表已连接"
    } else {
        "未连接手表，数据未更新"
    };
    let compact = layout == Layout::Phone;
    let summary = if compact { col(4) } else { row(16) };
    let summary = summary
        .prop("data-connection-summary", "true")
        .child(
            text(format!("● {label}"))
                .style(
                    "color",
                    if state.connected {
                        p::SECONDARY
                    } else {
                        p::AMBER
                    },
                )
                .style("flex-grow", "1"),
        )
        .child(muted(format!(
            "快照时间 · {} · 来源 · {}",
            time(state, Some(state.display_snapshot().captured_at)),
            state.display_snapshot().source
        )));
    let mut bar = col(6)
        .style("background", p::INSET)
        .style("padding", if compact { "6px 12px" } else { "8px 24px" })
        .style("flex-shrink", "0")
        .prop("role", "status")
        .prop("aria-label", "连接与快照状态")
        .child(summary);
    if let Some(id) = &state.viewed_backup_id {
        let mut preview =
            col(8).child(text(format!("正在查看备份快照 · {id}")).style("color", p::AMBER));
        if state.confirmation.is_none() && state.selected_item().is_none() {
            preview = preview.child(button("返回当前快照", "preview:exit", false));
        }
        bar = bar.child(preview);
    }
    bar
}
fn navigation(state: &AppState, layout: Layout) -> Node {
    let active = match state.page {
        Page::Matches | Page::MatchDetail => "matches",
        Page::Services => "services",
        Page::Settings => "settings",
    };
    let entries = [
        ("01", "战绩", "matches"),
        ("02", "特勤处", "services"),
        ("03", "设置", "settings"),
    ];
    if layout == Layout::Phone {
        return grid(3, 6)
            .style("padding", "8px 12px")
            .style("background", p::INSET)
            .style("flex-shrink", "0")
            .prop("aria-label", "底部主导航")
            .prop("data-navigation", "bottom")
            .children(
                entries
                    .into_iter()
                    .map(|(_, label, id)| button(label, format!("nav:{id}"), active == id)),
            );
    }
    let desktop = layout == Layout::Desktop;
    let mut nav = col(8)
        .style("width", if desktop { "156px" } else { "72px" })
        .style("padding", if desktop { "24px 12px" } else { "20px 6px" })
        .style("flex-shrink", "0")
        .style("background", p::INSET)
        .prop("aria-label", "左侧主导航")
        .prop(
            "data-navigation",
            if desktop { "sidebar" } else { "compact" },
        );
    if desktop {
        nav = nav.child(muted("同步工作台 / V4").style("margin-bottom", "16px"));
    }
    for (index, label, id) in entries {
        nav = nav.child(
            button(
                if desktop {
                    format!("{index}  {label}   →")
                } else {
                    label.into()
                },
                format!("nav:{id}"),
                active == id,
            )
            .style("padding-left", "4px")
            .style("padding-right", "4px"),
        );
    }
    if desktop {
        nav = nav.child(
            col(8)
                .style("margin-top", "28px")
                .child(muted("四页工作流"))
                .child(muted("战绩 → 对局详情"))
                .child(muted("特勤处 · 本地设置"))
                .child(muted("LOCAL / DEMO").style("color", p::AMBER)),
        );
    }
    nav
}
fn context_panel(state: &AppState) -> Node {
    let snapshot = &state.display_snapshot();
    col(16)
        .style("width", "224px")
        .style("flex-shrink", "0")
        .prop("aria-label", "快照上下文")
        .child(
            panel()
                .child(muted("当前快照 / INTEL").style("color", p::AMBER))
                .child(text(&snapshot.player.name).style("font-size", "20px"))
                .child(muted(format!("干员等级  Lv.{}", snapshot.player.level)))
                .child(metric("哈夫币", money(snapshot.player.funds), true))
                .child(muted(format!(
                    "仓库占用 {}/{}",
                    snapshot.player.warehouse_used, snapshot.player.warehouse_capacity
                )))
                .child(source_label(&snapshot.source)),
        )
        .child(
            panel()
                .child(muted("档案索引"))
                .child(text(format!("{} 场对局", snapshot.matches.len())))
                .child(text(format!("{} 类库存物品", snapshot.inventory.len())))
                .child(text(format!("{} 份本地备份", state.data.backups.len())))
                .child(muted(format!(
                    "游戏版本 · {}",
                    snapshot.source_game_version
                ))),
        )
        .child(
            col(8)
                .child(muted("演示工作流").style("color", p::AMBER))
                .child(muted("浏览战绩与伤害 → 检查库存 → 管理本地快照")),
        )
}

fn matches_page(state: &AppState, layout: Layout) -> Node {
    let records = &state.display_snapshot().matches;
    let wins = records.iter().filter(|record| success(record)).count();
    let kills = records
        .iter()
        .map(|record| u64::from(record.kills))
        .sum::<u64>();
    let value = records
        .iter()
        .filter(|record| record.mode == "operations")
        .fold(0_u64, |sum, record| sum.saturating_add(record.loot_value));
    let mut page = col(18)
        .child(section_title(
            "COMBAT ARCHIVE / 01",
            "战绩总览",
            "行动轨迹、撤离收获与战场表现",
        ))
        .child(
            grid(if layout == Layout::Desktop { 4 } else { 2 }, 10)
                .prop("aria-label", "战绩统计")
                .child(metric("档案对局", records.len().to_string(), false))
                .child(metric("成功 / 胜利", wins.to_string(), false))
                .child(metric("累计击败", kills.to_string(), false))
                .child(metric("烽火带出货值", money(value), true)),
        )
        .child(
            grid(3, 6).prop("aria-label", "战绩模式筛选").children(
                [
                    ("全部", "all"),
                    ("烽火地带", "operations"),
                    ("全面战场", "battlefield"),
                ]
                .into_iter()
                .map(|(label, id)| {
                    button(label, format!("filter:{id}"), state.history_filter == id)
                }),
            ),
        );
    let filtered = state.filtered_matches();
    page = page.child(
        row(8)
            .child(text("最近行动记录").style("flex-grow", "1"))
            .child(muted(format!("{} 条档案", filtered.len()))),
    );
    if filtered.is_empty() {
        page = page.child(empty(
            "当前筛选下暂无对局",
            "选择其他模式查看快照中的行动记录。",
        ));
    }
    page.child(
        grid(columns(layout), 12)
            .prop("aria-label", "对局列表")
            .children(filtered.into_iter().map(|record| match_card(state, record))),
    )
}
fn match_card(state: &AppState, record: &MatchRecord) -> Node {
    let value = if record.mode == "battlefield" {
        format!("{} 分", record.score)
    } else {
        format!("{} 货值", money(record.loot_value))
    };
    el("button")
        .event(format!("match:{}", record.id))
        .prop(
            "aria-label",
            format!("查看对局 {} {}", record.id, record.map_name),
        )
        .prop("data-match-id", &record.id)
        .style("display", "flex")
        .style("flex-direction", "column")
        .style("gap", "12px")
        .style("min-height", "120px")
        .style("min-width", "44px")
        .style("padding", "14px")
        .style("background", p::PANEL)
        .style("border", format!("1px solid {}", p::LINE))
        .style("border-radius", "4px")
        .style("color", p::TEXT)
        .child(
            grid(3, 8)
                .prop("data-line", "result-value-kills")
                .child(
                    text(outcome_label(record))
                        .style("color", if success(record) { "#9ed7af" } else { p::AMBER }),
                )
                .child(text(value))
                .child(muted(format!("{} 击败", record.kills))),
        )
        .child(
            text(format!(
                "{} · {} · {}",
                mode_label(record),
                record.map_name,
                record.difficulty
            ))
            .style("font-size", "13px")
            .style("color", p::SECONDARY)
            .prop("data-line", "mode-map-difficulty"),
        )
        .child(
            muted(format!("{}   ↗ 详情", time(state, record.started_at)))
                .prop("data-line", "started-at"),
        )
}
fn empty(title: &str, description: &str) -> Node {
    panel().child(text(title)).child(muted(description))
}

fn match_detail(state: &AppState, layout: Layout) -> Node {
    let Some(record) = state.selected_match() else {
        return col(16)
            .child(button("← 返回战绩", "back:matches", false))
            .child(empty(
                "请选择一场对局",
                "从战绩列表进入，即可查看本场伤害与战利品。",
            ));
    };
    let mut page = col(18)
        .child(button("← 战绩 / 返回列表", "back:matches", false))
        .child(section_title(
            "COMBAT REPORT / 详情",
            &format!("{} · {}", record.map_name, outcome_label(record)),
            format!(
                "{} · {} · {}",
                mode_label(record),
                record.difficulty,
                time(state, record.started_at)
            ),
        ))
        .child(source_label(&record.source));
    let stats = if record.mode == "battlefield" {
        grid(2, 10)
            .prop("aria-label", "全面战场统计")
            .child(metric("战场得分", record.score.to_string(), true))
            .child(metric(
                "击败 / 阵亡",
                format!("{} / {}", record.kills, record.deaths),
                false,
            ))
            .child(metric(
                "占领 / 救援",
                format!("{} / {}", record.captures, record.revives),
                false,
            ))
            .child(metric("经验获取", format!("+{} XP", record.xp_gain), false))
    } else {
        grid(2, 10)
            .prop("aria-label", "烽火地带统计")
            .child(metric("本场货值", money(record.loot_value), true))
            .child(metric("击败", record.kills.to_string(), false))
            .child(metric(
                "行动时长",
                format!(
                    "{}分{:02}秒",
                    record.duration_seconds / 60,
                    record.duration_seconds % 60
                ),
                false,
            ))
            .child(metric("经验获取", format!("+{} XP", record.xp_gain), false))
    };
    page = page.child(stats);
    let mut loot = panel()
        .prop("aria-label", "战利品明细")
        .child(text("带出物品 / 战利品").style("font-size", "18px"))
        .child(muted("按快照记录展示；安全箱保留物品单独标注。"));
    if record.loot.is_empty() {
        loot = loot.child(muted("本场快照未记录带出物品。"));
    }
    loot = loot.child(
        grid(if layout == Layout::Desktop { 2 } else { 1 }, 10)
            .children(record.loot.iter().map(item_card)),
    );
    page.child(
        grid(columns(layout), 16)
            .child(damage_panel(record))
            .child(loot),
    )
}

// Canonical game part codes are head/chest/abdomen/arm/leg. Aliases keep older snapshots legible.
const PARTS: [(&str, &str); 5] = [
    ("head", "头部"),
    ("chest", "胸部"),
    ("abdomen", "腹部"),
    ("arm", "手臂"),
    ("leg", "腿部"),
];
fn part_is(value: &str, part: &str) -> bool {
    match part {
        "head" => matches!(value, "head" | "头部" | "头"),
        "chest" => matches!(value, "chest" | "thorax" | "胸部" | "胸"),
        "abdomen" => matches!(value, "abdomen" | "stomach" | "腹部" | "腹"),
        "arm" => matches!(value, "arm" | "arms" | "手臂" | "双臂"),
        "leg" => matches!(value, "leg" | "legs" | "腿部" | "双腿"),
        _ => false,
    }
}
fn part_damage<'a>(record: &'a MatchRecord, part: &str) -> Option<&'a DamagePart> {
    record
        .damage
        .iter()
        .find(|damage| part_is(&damage.part, part))
}
fn number(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".into())
}
fn part_color(record: &MatchRecord, part: &str) -> &'static str {
    match part_damage(record, part).and_then(|damage| damage.damage_taken) {
        Some(100..) => p::RED,
        Some(1..=99) => "#805c16",
        _ => "#48514d",
    }
}
fn body_block(record: &MatchRecord, part: &str, width: u32, height: u32) -> Node {
    el("div")
        .style("width", format!("{width}px"))
        .style("height", format!("{height}px"))
        .style("background", part_color(record, part))
        .style("border", format!("1px solid {}", p::SECONDARY))
        .style("border-radius", "3px")
}
fn body_diagram(record: &MatchRecord) -> Node {
    col(5)
        .style("align-items", "center")
        .style("padding", "12px")
        .style("background", p::INSET)
        .prop("aria-label", "五部位人体伤害示意图")
        .child(body_block(record, "head", 34, 32).prop("data-body-part", "head"))
        .child(
            row(5)
                .style("align-items", "flex-start")
                .child(body_block(record, "arm", 18, 88).prop("data-body-part", "arm"))
                .child(
                    col(5)
                        .child(body_block(record, "chest", 54, 48).prop("data-body-part", "chest"))
                        .child(
                            body_block(record, "abdomen", 54, 34).prop("data-body-part", "abdomen"),
                        ),
                )
                .child(body_block(record, "arm", 18, 88).prop("aria-hidden", "true")),
        )
        .child(
            row(6)
                .prop("data-body-part", "leg")
                .child(body_block(record, "leg", 24, 66))
                .child(body_block(record, "leg", 24, 66)),
        )
        .child(muted("累计承伤热区"))
}
fn damage_panel(record: &MatchRecord) -> Node {
    let mut panel = panel()
        .prop("aria-label", "五部位伤害详情")
        .child(text("身体状态 / DAMAGE").style("font-size", "18px"))
        .child(body_diagram(record))
        .child(muted("累计承伤与结束伤势分别记录；— 表示快照未提供。"));
    let header = grid(3, 6)
        .child(muted("部位"))
        .child(muted("累计承伤"))
        .child(muted("结束伤势"));
    panel = panel.child(header);
    for (part, label) in PARTS {
        let damage = part_damage(record, part);
        panel = panel.child(
            grid(3, 6)
                .style("padding", "8px 0")
                .prop("data-damage-part", part)
                .prop("aria-label", format!("{label}承伤与伤势"))
                .child(text(label))
                .child(
                    text(number(damage.and_then(|damage| damage.damage_taken)))
                        .prop("data-measure", "damage-taken"),
                )
                .child(
                    text(number(damage.and_then(|damage| damage.injury_at_end)))
                        .prop("data-measure", "injury-at-end")
                        .style("color", p::AMBER),
                ),
        );
    }
    panel
}

fn services_page(state: &AppState, layout: Layout) -> Node {
    let player = &state.display_snapshot().player;
    let mut page = col(18)
        .child(section_title(
            "SPECIAL SERVICES / 02",
            "特勤处",
            format!(
                "{} · 干员等级 Lv.{} · 累计完成 {} 次行动",
                player.name, player.level, player.completed_runs
            ),
        ))
        .child(
            grid(2, 10)
                .child(metric("可用哈夫币", money(player.funds), true))
                .child(metric(
                    "仓库容量",
                    format!("{} / {}", player.warehouse_used, player.warehouse_capacity),
                    false,
                )),
        )
        .child(
            grid(3, 6).prop("aria-label", "特勤处标签页").children(
                [
                    ("库存", "inventory"),
                    ("设施", "facilities"),
                    ("部门任务", "missions"),
                ]
                .into_iter()
                .map(|(label, id)| {
                    button(label, format!("services:{id}"), state.services_tab == id)
                }),
            ),
        );
    page = page.child(match state.services_tab.as_str() {
        "facilities" => facilities_page(state, layout),
        "missions" => missions_page(state, layout),
        _ => inventory_page(state, layout),
    });
    page
}
fn inventory_page(state: &AppState, layout: Layout) -> Node {
    let search = el("input")
        .event("query:inventory")
        .prop("value", &state.inventory_query)
        .prop("placeholder", "搜索物品名称")
        .prop("aria-label", "搜索库存物品")
        .style("min-height", "44px")
        .style("min-width", "44px")
        .style("background", p::INSET)
        .style("color", p::TEXT)
        .style("padding", "8px 12px")
        .style("border", format!("1px solid {}", p::LINE))
        .style("border-radius", "3px");
    let mut page = col(14)
        .prop("data-services-tab", "inventory")
        .child(search)
        .child(muted("品质筛选"))
        .child(
            grid(if layout == Layout::Phone { 4 } else { 7 }, 6).children(
                ["all", "白", "绿", "蓝", "紫", "金", "红"]
                    .into_iter()
                    .map(|quality| {
                        button(
                            if quality == "all" { "全部" } else { quality },
                            format!("quality:{quality}"),
                            state.inventory_quality == quality,
                        )
                        .style(
                            "border",
                            format!(
                                "1px solid {}",
                                if quality == "all" {
                                    p::LINE
                                } else {
                                    p::quality(quality)
                                }
                            ),
                        )
                    }),
            ),
        )
        .child(muted("物品分类"))
        .child(
            grid(if layout == Layout::Phone { 3 } else { 6 }, 6).children(
                ["all", "装备", "弹药", "药品", "贵重品", "房卡"]
                    .into_iter()
                    .map(|category| {
                        button(
                            if category == "all" {
                                "全部"
                            } else {
                                category
                            },
                            format!("category:{category}"),
                            state.inventory_category == category,
                        )
                    }),
            ),
        );
    let items = state.filtered_inventory();
    page = page.child(muted(format!("库存物品 · {} 类", items.len())));
    if items.is_empty() {
        page = page.child(empty("没有匹配的物品", "调整名称、品质或分类后重新查看。"));
    }
    page.child(
        grid(columns(layout), 10)
            .prop("aria-label", "库存物品网格")
            .children(items.into_iter().map(item_card)),
    )
}
fn item_card(item: &Item) -> Node {
    el("button")
        .event(format!("item:{}", item.id))
        .prop("aria-label", format!("物品详情 {}", item.name))
        .prop("data-item-id", &item.id)
        .style("display", "flex")
        .style("flex-direction", "column")
        .style("gap", "9px")
        .style("min-height", "112px")
        .style("min-width", "44px")
        .style("padding", "12px")
        .style("background", p::INSET)
        .style("color", p::TEXT)
        .style("border", format!("1px solid {}", p::quality(&item.quality)))
        .style("border-radius", "3px")
        .child(
            row(6)
                .child(badge(
                    format!("{}色", item.quality),
                    p::quality(&item.quality),
                ))
                .child(muted(&item.category)),
        )
        .child(text(&item.name).style("font-size", "15px"))
        .child(
            text(format!(
                "×{} · 总值 {}",
                item.quantity,
                money(total_value(item))
            ))
            .style("color", p::AMBER),
        )
        .child(muted(format!(
            "{} · {} 格{}",
            compartment_label(&item.compartment),
            item.slots,
            if item.locked { " · 已锁定" } else { "" }
        )))
}
fn facilities_page(state: &AppState, layout: Layout) -> Node {
    let mut page = col(14)
        .prop("data-services-tab", "facilities")
        .child(section_title(
            "BASE INFRASTRUCTURE",
            "后勤设施",
            "仓库与训练设施的当前等级和生效加成",
        ));
    if state.display_snapshot().facilities.is_empty() {
        page = page.child(empty("设施档案待补充", "当前快照中的设施将在此展示。"));
    }
    page.child(
        grid(columns(layout), 12).children(state.display_snapshot().facilities.iter().map(
            |facility| {
                let max = facility.max_level.max(1);
                panel()
                    .prop("data-facility-id", &facility.id)
                    .child(
                        row(8)
                            .child(badge("设施", p::GREEN))
                            .child(text(&facility.name).style("font-size", "18px")),
                    )
                    .child(
                        text(format!("Lv.{} / {}", facility.level, facility.max_level))
                            .style("color", p::AMBER),
                    )
                    .child(muted(&facility.description))
                    .child(
                        el("progress")
                            .prop("value", facility.level.min(max))
                            .prop("max", max)
                            .prop("aria-label", format!("{}等级进度", facility.name))
                            .style("height", "6px")
                            .style("background", p::INSET)
                            .style("color", p::GREEN),
                    )
                    .child(text(&facility.effect))
            },
        )),
    )
}
fn missions_page(state: &AppState, layout: Layout) -> Node {
    let mut page = col(14)
        .prop("data-services-tab", "missions")
        .child(section_title(
            "DEPARTMENT TASKS",
            "部门任务",
            "规划演示 · 依据快照展示任务进度与奖励",
        ))
        .child(badge("规划演示", "#805c16"))
        .child(
            grid(3, 6).children(
                [
                    ("全部", "all"),
                    ("进行中", "pending"),
                    ("已完成", "completed"),
                ]
                .into_iter()
                .map(|(label, filter)| {
                    button(
                        label,
                        format!("mission:{filter}"),
                        state.mission_filter == filter,
                    )
                }),
            ),
        );
    let missions = state.filtered_missions();
    if missions.is_empty() {
        page = page.child(empty("当前筛选下暂无任务", "切换筛选查看其他部门任务。"));
    }
    page.child(
        grid(columns(layout), 12).children(missions.into_iter().map(|mission| {
            let complete = mission.progress >= mission.target;
            panel()
                .prop("data-mission-id", &mission.id)
                .child(
                    row(6)
                        .child(badge(&mission.department, p::GREEN))
                        .child(badge(if complete { "已完成" } else { "进行中" }, p::INSET)),
                )
                .child(text(&mission.title).style("font-size", "18px"))
                .child(muted(&mission.description))
                .child(text(format!(
                    "进度  {} / {}",
                    mission.progress, mission.target
                )))
                .child(
                    el("progress")
                        .prop("value", mission.progress.min(mission.target.max(1)))
                        .prop("max", mission.target.max(1))
                        .prop("aria-label", format!("{}任务进度", mission.title))
                        .style("height", "6px")
                        .style("color", p::GREEN)
                        .style("background", p::INSET),
                )
                .child(text(format!("奖励 · {}", mission.reward)).style("color", p::AMBER))
                .child(source_label(&mission.source))
        })),
    )
}

fn toggle(label: &str, id: &str, selected: bool) -> Node {
    el("button")
        .text(format!(
            "{}  {}",
            if selected {
                "● 已开启"
            } else {
                "○ 已关闭"
            },
            label
        ))
        .event(id)
        .prop("type", "button")
        .prop("role", "switch")
        .prop("selected", selected)
        .prop("value", selected)
        .prop("aria-checked", selected)
        .prop("aria-label", label)
        .style("display", "flex")
        .style("align-items", "center")
        .style("min-height", "44px")
        .style("min-width", "44px")
        .style("padding", "8px 12px")
        .style("border", format!("1px solid {}", p::LINE))
        .style("border-radius", "3px")
        .style("background", if selected { p::GREEN } else { p::INSET })
        .style("color", p::TEXT)
}
fn settings_page(state: &AppState, layout: Layout) -> Node {
    let mut page = col(18).child(section_title(
        "LOCAL CONTROL / 03",
        "设置与本地备份",
        "管理模拟连接、快照保留与本地备份",
    ));
    let device = panel()
        .prop("aria-label", "设备演示卡")
        .child(
            row(8)
                .child(badge("DEMO DEVICE", p::INSET))
                .child(text("手表连接").style("font-size", "18px")),
        )
        .child(text(if state.connected {
            "演示连接 · 模拟手表"
        } else {
            "等待连接 · 模拟手表"
        }))
        .child(muted("通过下方开关体验连接状态与界面反馈。"))
        .child(toggle("模拟设备连接", "device:toggle", state.connected))
        .child(source_label(&state.display_snapshot().source));
    let backup_preferences = panel()
        .prop("aria-label", "备份偏好")
        .child(text("备份策略").style("font-size", "18px"))
        .child(toggle(
            "启动时自动备份",
            "settings:auto-backup",
            state.data.settings.auto_backup_on_launch,
        ))
        .child(muted("保留份数"))
        .child(
            el("input")
                .event("settings:retention-input")
                .prop("type", "number")
                .prop("value", &state.retention_input)
                .prop("aria-label", "本地备份保留份数")
                .prop("min", "1")
                .style("min-height", "44px")
                .style("min-width", "44px")
                .style("padding", "8px 12px")
                .style("background", p::INSET)
                .style("color", p::TEXT)
                .style("border", format!("1px solid {}", p::LINE))
                .style("border-radius", "3px"),
        )
        .child(button("应用保留份数", "settings:retention-apply", false))
        .child(muted(format!(
            "当前策略 · 保留 {} 份",
            state.data.settings.retention_count
        )));
    page = page
        .child(
            grid(columns(layout), 14)
                .child(device)
                .child(backup_preferences),
        )
        .child(
            panel()
                .child(
                    row(8)
                        .child(
                            text("本地快照库")
                                .style("font-size", "18px")
                                .style("flex-grow", "1"),
                        )
                        .child(badge(format!("{} 份", state.data.backups.len()), p::INSET)),
                )
                .child(muted("每份快照包含玩家资料、战绩、库存与任务进度。"))
                .child(button("＋ 创建本地备份", "backup:create", true)),
        );
    if state.data.backups.is_empty() {
        page = page.child(empty(
            "快照库已准备就绪",
            "创建第一份本地备份，保存当前演示数据。",
        ));
    }
    page.child(
        col(12)
            .prop("aria-label", "本地备份列表")
            .children(state.data.backups.iter().map(|backup| {
                panel()
                    .prop("data-backup-id", &backup.id)
                    .child(text(time(state, Some(backup.created_at))).style("font-size", "18px"))
                    .child(muted(format!("{} · {}", backup.device_name, backup.id)))
                    .child(source_label(&backup.source))
                    .child(muted(format!(
                        "{} 场对局 · {} 类库存 · 哈夫币 {}",
                        backup.snapshot.matches.len(),
                        backup.snapshot.inventory.len(),
                        money(backup.snapshot.player.funds)
                    )))
                    .child(
                        grid(if layout == Layout::Phone { 2 } else { 4 }, 6).children(
                            [
                                ("查看", "view"),
                                ("恢复", "restore"),
                                ("导出", "export"),
                                ("删除", "delete"),
                            ]
                            .into_iter()
                            .map(|(label, action)| {
                                button(label, format!("backup:{action}:{}", backup.id), false)
                                    .prop("aria-label", format!("{label}备份 {}", backup.id))
                                    .style(
                                        "color",
                                        if action == "delete" {
                                            p::AMBER
                                        } else {
                                            p::TEXT
                                        },
                                    )
                            }),
                        ),
                    )
            })),
    )
}

fn modal_shell(label: &str, layout: Layout) -> Node {
    let content = col(0)
        .style("position", "absolute")
        .style("top", "0")
        .style("left", "0")
        .style("right", "0")
        .style("bottom", "0")
        .style("z-index", "100")
        .style("background", p::BACKGROUND)
        .style(
            "padding",
            if layout == Layout::Phone {
                "20px 12px"
            } else {
                "32px"
            },
        )
        .prop("role", "dialog")
        .prop("aria-modal", "true")
        .prop("aria-label", label);
    content.child(
        el("scroll-area")
            .style("flex-grow", "1")
            .style("min-height", "0")
            .prop("aria-label", format!("{label}滚动内容")),
    )
}
fn with_modal_content(mut shell: Node, content: Node) -> Node {
    if let Some(scroll) = shell.children.first_mut() {
        scroll.children.push(
            row(0)
                .style("align-items", "flex-start")
                .style("justify-content", "center")
                .child(content.style("width", "100%").style("max-width", "560px")),
        );
    }
    shell
}
fn item_modal(item: &Item, layout: Layout) -> Node {
    let content = panel()
        .child(muted("ITEM INTELLIGENCE / 物品档案").style("color", p::AMBER))
        .child(badge(
            format!("{}色品质 · {}", item.quality, item.category),
            p::quality(&item.quality),
        ))
        .child(heading(&item.name))
        .child(
            grid(2, 10)
                .child(metric("持有数量", item.quantity.to_string(), false))
                .child(metric("单位价值", money(item.unit_value), false))
                .child(metric("物品总值", money(total_value(item)), true))
                .child(metric("占用格数", item.slots.to_string(), false)),
        )
        .child(text(format!(
            "存放位置 · {}",
            compartment_label(&item.compartment)
        )))
        .child(text(format!(
            "锁定状态 · {}",
            if item.locked { "已锁定" } else { "可用" }
        )))
        .child(text(format!(
            "耐久 · {} / {}",
            number(item.durability),
            number(item.max_durability)
        )))
        .child(button("关闭物品详情", "item:close", false));
    with_modal_content(modal_shell("物品详情", layout), content)
}
fn confirmation_modal(state: &AppState, confirmation: &Confirmation, layout: Layout) -> Node {
    let (title, description, accept, target) = match confirmation {
        Confirmation::Restore(id) => (
            "恢复本地快照",
            "当前展示的数据将更新为所选备份，设置与备份库继续保留。".into(),
            "确认恢复",
            Some(id),
        ),
        Confirmation::DeleteBackup(id) => (
            "删除这份备份",
            "确认后将从本地快照库移除此记录，当前展示的快照继续保留。".into(),
            "确认删除",
            Some(id),
        ),
        Confirmation::SetRetention(count) => (
            "更新备份保留策略",
            format!("保留最近 {count} 份本地备份，超出份数的较早记录将清理。"),
            "确认应用",
            None,
        ),
    };
    let mut content = panel()
        .child(badge("本地操作确认", "#805c16"))
        .child(heading(title))
        .child(text(description))
        .child(muted("操作范围 · 本插件的本地快照与备份库"));
    if let Some(id) = target {
        content = content.child(text(format!("备份 ID · {id}")));
        if let Some(backup) = state.data.backups.iter().find(|backup| &backup.id == id) {
            content = content
                .child(text(format!(
                    "备份时间 · {}",
                    time(state, Some(backup.created_at))
                )))
                .child(source_label(&backup.source))
                .child(text(format!("玩家 · {}", backup.snapshot.player.name)));
        }
    }
    content = content.child(
        grid(if layout == Layout::Phone { 1 } else { 2 }, 8)
            .child(button("取消，返回", "confirm:cancel", false))
            .child(button(accept, "confirm:accept", true)),
    );
    with_modal_content(modal_shell("本地备份操作确认", layout), content)
}
