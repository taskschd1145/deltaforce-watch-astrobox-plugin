//! Serializable UI tree, shared by the V4 adapter and structural/visual validation.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub tag: String,
    pub text: Option<String>,
    pub styles: BTreeMap<String, String>,
    pub props: BTreeMap<String, String>,
    pub event_id: Option<String>,
    pub children: Vec<Node>,
}
impl Node {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.into(),
            text: None,
            styles: BTreeMap::new(),
            props: BTreeMap::new(),
            event_id: None,
            children: Vec::new(),
        }
    }
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn style(mut self, key: &str, value: impl ToString) -> Self {
        self.styles.insert(key.into(), value.to_string());
        self
    }
    pub fn prop(mut self, key: &str, value: impl ToString) -> Self {
        self.props.insert(key.into(), value.to_string());
        self
    }
    pub fn event(mut self, id: impl Into<String>) -> Self {
        self.event_id = Some(id.into());
        self
    }
    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }
    pub fn children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(nodes);
        self
    }
}

/// Lower semantic grids to public Host Flex builders. The shipped AstroBox renderer
/// may render GRID as an ordinary block; equal shrinking percentage cells work across
/// native hosts and preserve all business nodes/events without private CSS injection.
pub fn lower_native_layout(mut node: Node) -> Node {
    node.children = node.children.into_iter().map(lower_native_layout).collect();
    if node
        .styles
        .get("display")
        .is_some_and(|value| value == "grid")
    {
        let columns = node
            .styles
            .get("grid-template-columns")
            .and_then(|value| value.strip_prefix("repeat("))
            .and_then(|value| value.split(',').next())
            .and_then(|value| value.trim().parse::<usize>().ok())
            .filter(|count| (1..=12).contains(count))
            .unwrap_or(1);
        let spacing = node
            .styles
            .get("gap")
            .cloned()
            .unwrap_or_else(|| "0px".into());
        node.tag = "div".into();
        node.styles.insert("display".into(), "flex".into());
        node.styles.insert("flex-direction".into(), "column".into());
        node.styles.remove("grid-template-columns");
        let children = std::mem::take(&mut node.children);
        let mut pending = children.into_iter();
        loop {
            let chunk: Vec<_> = pending.by_ref().take(columns).collect();
            if chunk.is_empty() {
                break;
            }
            let mut row = Node::new("div")
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("width", "100%")
                .style("min-width", "0")
                .style("gap", &spacing);
            let count = chunk.len();
            for child in chunk {
                row = row.child(
                    Node::new("div")
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .style("width", "100%")
                        .style("flex-grow", "1")
                        .style("flex-shrink", "1")
                        .style("min-width", "0")
                        .child(child),
                );
            }
            for _ in count..columns {
                row = row.child(
                    Node::new("div")
                        .style("width", "100%")
                        .style("flex-grow", "1")
                        .style("flex-shrink", "1")
                        .style("min-width", "0"),
                );
            }
            node.children.push(row);
        }
    }
    node
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Phone,
    Tablet,
    Desktop,
}
impl Layout {
    pub fn for_width(width: u32) -> Self {
        if width >= 1024 {
            Self::Desktop
        } else if width >= 640 {
            Self::Tablet
        } else {
            Self::Phone
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Phone => "手机布局",
            Self::Tablet => "平板布局",
            Self::Desktop => "电脑布局",
        }
    }
}
