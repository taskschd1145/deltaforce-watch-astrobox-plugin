//! Emit the same pure UI tree used by the V4 host for deterministic visual validation.
use deltaforce_watch_astrobox_plugin::{contracts::*,demo,ui,view};
use std::{fs,path::PathBuf};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let directory=PathBuf::from("docs/validation/previews");fs::create_dir_all(&directory)?;
    for (label,width,height) in [("phone",390,844),("tablet",768,1024),("desktop",1440,900)] {
        let mut state=AppState::new(demo::initial_data());
        for (page_name,page) in [("matches",Page::Matches),("detail",Page::MatchDetail),("services",Page::Services),("settings",Page::Settings)] {
            state.page=page;
            if page==Page::MatchDetail {state.selected_match_id=state.data.snapshot.matches.iter().find(|r|r.mode=="operations"&&r.outcome=="killed").map(|r|r.id.clone());}
            let tree=view::lower_native_layout(ui::render(&state,width,height));
            let payload=serde_json::json!({"width":width,"height":height,"tree":tree});
            fs::write(directory.join(format!("{label}-{page_name}.json")),serde_json::to_vec(&payload)?)?;
        }
    }
    println!("Emitted twelve deterministic previews from the native UI render tree.");Ok(())
}
