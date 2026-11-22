//! Native AstroBox API Level 4 adapter. Device communication is simulated in the pure state layer.
use crate::{contracts::*, demo, export, state, store, ui, view::Node};
use std::{
    cell::RefCell,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

wit_bindgen::generate!({ path: "wit", world: "psys-world-v4", generate_all });
use astrobox::psys_host_v4::{dialog, os, timer, ui as host_ui};
use exports::astrobox::psys_plugin_v4::{event, lifecycle};

struct Runtime {
    state: AppState,
    element_id: Option<String>,
    size: (u32, u32),
    resize_timer: Option<u64>,
}
impl Runtime {
    fn new() -> Self {
        let (data, error) = match store::load(Path::new(".")) {
            Ok(data) => (data, None),
            Err(error) => (demo::initial_data(), Some(error)),
        };
        let mut state = AppState::new(data);
        if let Some(error) = error {
            state.notice =
                format!("本地数据需要检查：{error}。现已载入示例数据，原文件保持可供检查。");
        }
        Self {
            state,
            element_id: None,
            size: (0, 0),
            resize_timer: None,
        }
    }
}
thread_local! { static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new()); }
struct Synchronizer;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| i64::try_from(v.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(DEMO_CAPTURED_AT)
}
fn px(value: &str) -> Option<u32> {
    value.trim().trim_end_matches("px").parse().ok()
}
fn sides(value: &str) -> Option<[u32; 4]> {
    let parts: Vec<_> = value
        .split_whitespace()
        .map(px)
        .collect::<Option<Vec<_>>>()?;
    match parts.as_slice() {
        [a] => Some([*a; 4]),
        [a, b] => Some([*a, *b, *a, *b]),
        [a, b, c] => Some([*a, *b, *c, *b]),
        [a, b, c, d] => Some([*a, *b, *c, *d]),
        _ => None,
    }
}

/// Map the reviewed serializable view to the public Host builder API only.
fn host_node(node: Node) -> host_ui::Element {
    use host_ui::{Element, ElementType as T, Event as E, FlexDirection as F};
    let is_grid = node.styles.get("display").is_some_and(|v| v == "grid");
    let kind = match node.tag.as_str() {
        "button" => T::Button,
        "input" => T::Input,
        "select" => T::Select,
        "option" => T::Option,
        "p" => T::P,
        "span" => T::Span,
        "svg" => T::Svg,
        "scroll-area" => T::ScrollArea,
        "progress" => T::Progress,
        "checkbox" => T::Checkbox,
        "switch" => T::Switch,
        _ if is_grid => T::Grid,
        _ => T::Div,
    };
    let mut element = Element::new(kind, node.text.as_deref());
    // Grid/scroll primitives retain their native display/overflow behavior.
    // Other nodes use the game's explicit colors, spacing and shape.
    if !is_grid && node.tag != "scroll-area" {
        element = element.without_default_styles();
    }
    if is_grid { /* GRID supplies its native grid display role. */ }
    for (key, value) in &node.styles {
        match key.as_str() {
            "background" => element = element.bg(value),
            "color" => element = element.text_color(value),
            "font-size" => {
                if let Some(v) = px(value) {
                    element = element.size(v);
                }
            }
            "display" if value == "flex" => element = element.flex(),
            "flex-direction" => {
                element = element.flex_direction(match value.as_str() {
                    "row" => F::Row,
                    "row-reverse" => F::RowReverse,
                    "column-reverse" => F::ColumnReverse,
                    _ => F::Column,
                })
            }
            "align-items" => {
                element = match value.as_str() {
                    "center" => element.align_center(),
                    "flex-end" => element.align_end(),
                    _ => element.align_start(),
                }
            }
            "justify-content" => {
                element = match value.as_str() {
                    "center" => element.justify_center(),
                    "flex-end" => element.justify_end(),
                    _ => element.justify_start(),
                }
            }
            "width" => {
                if value == "100%" {
                    element = element.width_full();
                } else if value == "50%" {
                    element = element.width_half();
                } else if let Some(v) = px(value) {
                    element = element.width(v);
                }
            }
            "height" => {
                if value == "100%" {
                    element = element.height_full();
                } else if value == "50%" {
                    element = element.height_half();
                } else if let Some(v) = px(value) {
                    element = element.height(v);
                }
            }
            "min-width" => {
                if let Some(v) = px(value) {
                    element = element.min_width(v);
                }
            }
            "max-width" => {
                if let Some(v) = px(value) {
                    element = element.max_width(v);
                }
            }
            "min-height" => {
                if let Some(v) = px(value) {
                    element = element.min_height(v);
                }
            }
            "max-height" => {
                if let Some(v) = px(value) {
                    element = element.max_height(v);
                }
            }
            "padding" => {
                if let Some([t, r, b, l]) = sides(value) {
                    element = element
                        .padding_top(t)
                        .padding_right(r)
                        .padding_bottom(b)
                        .padding_left(l);
                }
            }
            "margin" => {
                if let Some([t, r, b, l]) = sides(value) {
                    element = element
                        .margin_top(t)
                        .margin_right(r)
                        .margin_bottom(b)
                        .margin_left(l);
                }
            }
            "padding-top" => {
                if let Some(v) = px(value) {
                    element = element.padding_top(v);
                }
            }
            "padding-right" => {
                if let Some(v) = px(value) {
                    element = element.padding_right(v);
                }
            }
            "padding-bottom" => {
                if let Some(v) = px(value) {
                    element = element.padding_bottom(v);
                }
            }
            "padding-left" => {
                if let Some(v) = px(value) {
                    element = element.padding_left(v);
                }
            }
            "margin-top" => {
                if let Some(v) = px(value) {
                    element = element.margin_top(v);
                }
            }
            "margin-right" => {
                if let Some(v) = px(value) {
                    element = element.margin_right(v);
                }
            }
            "margin-bottom" => {
                if let Some(v) = px(value) {
                    element = element.margin_bottom(v);
                }
            }
            "margin-left" => {
                if let Some(v) = px(value) {
                    element = element.margin_left(v);
                }
            }
            "border-radius" => {
                if let Some(v) = px(value) {
                    element = element.radius(v);
                }
            }
            "border" => {
                let fields: Vec<_> = value.split_whitespace().collect();
                if fields.len() == 3 {
                    if let Some(v) = px(fields[0]) {
                        element = element.border(v, fields[2]);
                    }
                }
            }
            "gap" => {
                if let Some(v) = px(value) {
                    element = element.gap(v);
                }
            }
            "grid-template-columns" => element = element.grid_template_columns(value),
            "flex-grow" => {
                if let Ok(v) = value.parse() {
                    element = element.flex_grow(v);
                }
            }
            "flex-shrink" => {
                if let Ok(v) = value.parse() {
                    element = element.flex_shrink(v);
                }
            }
            "position" => {
                element = if value == "absolute" {
                    element.absolute()
                } else {
                    element.relative()
                }
            }
            "top" => {
                if let Some(v) = px(value) {
                    element = element.top(v);
                }
            }
            "right" => {
                if let Some(v) = px(value) {
                    element = element.right(v);
                }
            }
            "bottom" => {
                if let Some(v) = px(value) {
                    element = element.bottom(v);
                }
            }
            "left" => {
                if let Some(v) = px(value) {
                    element = element.left(v);
                }
            }
            "z-index" => {
                if let Ok(v) = value.parse() {
                    element = element.z_index(v);
                }
            }
            "opacity" => {
                if let Ok(v) = value.parse() {
                    element = element.opacity(v);
                }
            }
            _ => {}
        }
    }
    for (key, value) in &node.props {
        element = element.prop(key, value);
        if key == "disabled" && value == "true" {
            element = element.disabled();
        }
    }
    if let Some(id) = &node.event_id {
        let event = match node.tag.as_str() {
            "input" | "textarea" => E::Input,
            "select" | "checkbox" | "switch" => E::Change,
            _ => E::Click,
        };
        element = element.on(event, id);
    }
    for child in node.children {
        element = element.child(host_node(child));
    }
    element
}
fn render_current() {
    let output = RUNTIME.with(|runtime| {
        let runtime = runtime.borrow();
        runtime.element_id.clone().map(|id| {
            (
                id,
                ui::render(&runtime.state, runtime.size.0, runtime.size.1),
            )
        })
    });
    if let Some((id, node)) = output {
        host_ui::render(&id, host_node(crate::view::lower_native_layout(node)));
    }
}
async fn read_dimensions() -> (u32, u32) {
    let size = host_ui::get_render_size().await;
    (size.width, size.height)
}
async fn export_record(id: &str) -> Result<String, String> {
    let record = RUNTIME
        .with(|runtime| {
            runtime
                .borrow()
                .state
                .data
                .backups
                .iter()
                .find(|b| b.id == id)
                .cloned()
        })
        .ok_or_else(|| "所选备份已变化，请重新选择。".to_string())?;
    let stamp = now();
    let bytes = export::export_backup(&record, stamp)?;
    export::verify_export(&bytes)?;
    let time = chrono::DateTime::from_timestamp_millis(stamp)
        .map(|v| v.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| stamp.to_string());
    let short_id: String = record
        .id
        .chars()
        .filter(|v| v.is_ascii_alphanumeric())
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let file_name = format!("deltaforce-sync-demo-{time}-{short_id}.json");
    let session = dialog::save_file_start(dialog::FilterConfig {
        multiple: false,
        extensions: vec!["json".into()],
        default_directory: String::new(),
        default_file_name: file_name,
    })
    .await
    .map_err(|e| format!("保存选择已结束：{e}"))?;
    for chunk in bytes.chunks(64 * 1024) {
        if let Err(error) = dialog::save_file_write_chunk(session.session_id, chunk.to_vec()).await
        {
            dialog::save_file_abort(session.session_id).await;
            return Err(format!("保存文件时遇到问题：{error}"));
        }
    }
    if let Err(error) = dialog::save_file_finish(session.session_id).await {
        dialog::save_file_abort(session.session_id).await;
        return Err(format!("完成文件保存时遇到问题：{error}"));
    }
    Ok(format!(
        "演示备份已导出：{} · {} 字节 · SHA-256 校验完成。",
        session.name,
        bytes.len()
    ))
}

impl lifecycle::Guest for Synchronizer {
    async fn on_load() {
        let zone = os::timezone_offset_minutes().await;
        RUNTIME.with(|runtime| runtime.borrow_mut().state.timezone_offset_minutes = zone);
        println!("三角洲行动同步器 0.1.0 · API Level 4 / WASI 3 · UI Demo");
    }
}
impl event::Guest for Synchronizer {
    async fn on_event(event_type: event::EventType, _payload: String) -> String {
        if matches!(event_type, event::EventType::Timer) {
            let mounted = RUNTIME.with(|runtime| runtime.borrow().element_id.is_some());
            if mounted {
                let size = read_dimensions().await;
                let changed = RUNTIME.with(|runtime| {
                    let mut rt = runtime.borrow_mut();
                    if rt.size == size {
                        false
                    } else {
                        rt.size = size;
                        true
                    }
                });
                if changed {
                    render_current();
                }
            }
        }
        String::new()
    }
    async fn on_ui_event(event_id: String, _event: host_ui::Event, payload: String) -> String {
        let effect = RUNTIME.with(|runtime| {
            let mut rt = runtime.borrow_mut();
            let mut candidate = rt.state.clone();
            let effect = state::dispatch(&mut candidate, &event_id, &payload, now());
            if effect.persist {
                if let Err(error) = store::save(Path::new("."), &candidate.data) {
                    rt.state.notice = format!("本地保存需要重试：{error}；当前快照与备份已保留。");
                    return Effect::default();
                }
            }
            rt.state = candidate;
            effect
        });
        render_current();
        if let Some(id) = effect.export_backup {
            let result = export_record(&id).await;
            RUNTIME.with(|runtime| {
                let mut rt = runtime.borrow_mut();
                rt.state.busy = false;
                rt.state.notice = match result {
                    Ok(message) => message,
                    Err(error) => format!("导出提示：{error}；插件内备份已保留。"),
                };
            });
            render_current();
        }
        String::new()
    }
    async fn on_ui_render(element_id: String) {
        let size = read_dimensions().await;
        RUNTIME.with(|runtime| {
            let mut rt = runtime.borrow_mut();
            rt.element_id = Some(element_id);
            rt.size = size;
            if rt.resize_timer.is_none() {
                rt.resize_timer = Some(timer::set_interval(500, "deltaforce-ui-resize"));
            }
        });
        render_current();
    }
    async fn on_card_render(card_id: String) {
        host_ui::render_to_text_card(
            &card_id,
            "三角洲行动同步器 · UI Demo · 请打开插件页查看战绩、特勤处与离线备份。",
        );
    }
}
export!(Synchronizer);
