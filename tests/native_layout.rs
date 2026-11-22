use deltaforce_watch_astrobox_plugin::{
    contracts::*,
    demo, ui,
    view::{Node, lower_native_layout},
};

fn events(node: &Node, output: &mut Vec<String>) {
    if let Some(id) = &node.event_id {
        output.push(id.clone());
    }
    for child in &node.children {
        events(child, output);
    }
}
fn check(node: &Node) {
    assert_ne!(node.styles.get("display").map(String::as_str), Some("grid"));
    assert!(!node.styles.contains_key("grid-template-columns"));
    for child in &node.children {
        check(child);
    }
}
#[test]
fn lowering_keeps_every_event_and_replaces_grid_with_public_flex_builders() {
    let mut state = AppState::new(demo::initial_data());
    for page in [
        Page::Matches,
        Page::Services,
        Page::Settings,
        Page::MatchDetail,
    ] {
        state.page = page;
        state.selected_match_id = Some(state.data.snapshot.matches[0].id.clone());
        for width in [0, 360, 390, 639, 640, 768, 1023, 1024, 1440] {
            let tree = ui::render(&state, width, 800);
            let mut before = Vec::new();
            events(&tree, &mut before);
            let lowered = lower_native_layout(tree);
            let mut after = Vec::new();
            events(&lowered, &mut after);
            assert_eq!(before, after);
            check(&lowered);
        }
    }
}
#[test]
fn native_rows_keep_equal_cell_count_and_shrinking_widths() {
    let grid = Node::new("div")
        .style("display", "grid")
        .style("grid-template-columns", "repeat(3, minmax(0, 1fr))")
        .children((0..5).map(|i| Node::new("button").event(format!("item:{i}"))));
    let lowered = lower_native_layout(grid);
    assert_eq!(lowered.children.len(), 2);
    for row in lowered.children {
        assert_eq!(row.children.len(), 3);
        for cell in row.children {
            assert_eq!(cell.styles.get("width").map(String::as_str), Some("100%"));
            assert_eq!(cell.styles.get("min-width").map(String::as_str), Some("0"));
            assert_eq!(
                cell.styles.get("flex-shrink").map(String::as_str),
                Some("1")
            );
        }
    }
}
