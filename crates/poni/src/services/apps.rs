//! Start-menu discovery for the Apps page. It runs once on request: no
//! registry walk, process watcher or timer remains alive while the page rests.

use crate::events::{AppEvent, EventSink};
use crate::state::LaunchApp;
use std::collections::BTreeMap;
use std::path::Path;

const LIMIT: usize = 300;

/// Read the current user's and all-users' Start Menu shortcuts off the UI
/// thread. A shortcut itself is launched later through the existing safe
/// absolute-path shell route, so no command line is ever reconstructed here.
pub fn refresh(sink: EventSink) {
    let _ = std::thread::Builder::new().name("poni-apps".into()).spawn(move || {
        sink.send(AppEvent::Apps(read()));
    });
}

fn read() -> Vec<LaunchApp> {
    let mut by_name = BTreeMap::new();
    for root in start_roots() {
        visit(&root, &mut by_name);
        if by_name.len() >= LIMIT { break; }
    }
    by_name.into_values().collect()
}

fn start_roots() -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    if let Some(p) = std::env::var_os("APPDATA") {
        roots.push(Path::new(&p).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Some(p) = std::env::var_os("ProgramData") {
        roots.push(Path::new(&p).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    roots
}

fn visit(dir: &Path, out: &mut BTreeMap<String, LaunchApp>) {
    if out.len() >= LIMIT { return; }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if out.len() >= LIMIT { break; }
        let path = entry.path();
        if path.is_dir() {
            visit(&path, out);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk") || e.eq_ignore_ascii_case("url")) {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::trim).filter(|s| !s.is_empty()) else { continue };
            let key = stem.to_lowercase();
            out.entry(key).or_insert_with(|| LaunchApp { name: stem.to_string(), path: path.to_string_lossy().into_owned() });
        }
    }
}

