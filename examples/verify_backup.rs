//! Verify an exported demo backup using the same strict decoder as the plugin.
use deltaforce_watch_astrobox_plugin::export;
fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Usage: verify_backup <export.json>");
        std::process::exit(2);
    };
    let result = std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| export::verify_export(&bytes));
    match result {
        Ok(backup) => println!(
            "{}",
            serde_json::json!({"result":"verified","backupId":backup.id,"source":backup.source,"matches":backup.snapshot.matches.len(),"inventoryItems":backup.snapshot.inventory.len(),"algorithm":"sha256"})
        ),
        Err(error) => {
            eprintln!("Backup verification failed: {error}");
            std::process::exit(1);
        }
    }
}
