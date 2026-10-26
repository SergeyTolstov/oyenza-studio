//! Windows Start-menu apps. The list is fetched once when this page opens and
//! filtered locally; it neither watches installed software nor runs at rest.

use crate::commands::Command;
use crate::gfx::TextStyle;
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, wid_n, Ui};
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 200.0;
const SEARCH_H: f32 = 34.0;
const ROW_H: f32 = 31.0;
const MAX_ROWS: usize = 5;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let search = Rect::new(area.x, area.y, area.w, SEARCH_H);
    ui.text_input_row(wid("apps.search"), search, &mut st.ui.apps_query, "Search installed apps", 80, st.accent, Some(Icon::App));
    if st.launch_apps.is_empty() {
        super::empty_state(ui, Rect::new(area.x, area.y + 42.0, area.w, HEIGHT - 42.0), Icon::App, "Loading apps", "Open this page to refresh your Start menu");
        if !st.ui.apps_loading {
            st.ui.apps_loading = true;
            st.cmd(Command::RefreshApps);
        }
        return;
    }
    let words: Vec<String> = st.ui.apps_query.split_whitespace().map(|s| s.to_lowercase()).collect();
    let matches: Vec<_> = st.launch_apps.iter().filter(|a| {
        let name = a.name.to_lowercase();
        words.iter().all(|w| name.contains(w))
    }).take(MAX_ROWS).cloned().collect();
    if matches.is_empty() {
        super::empty_state(ui, Rect::new(area.x, area.y + 42.0, area.w, HEIGHT - 42.0), Icon::App, "No matching apps", "Try a different name");
        return;
    }
    for (i, app) in matches.iter().enumerate() {
        let row = Rect::new(area.x, area.y + 42.0 + i as f32 * ROW_H, area.w, ROW_H - 2.0);
        let hit = ui.button(wid_n("apps.launch", i as u64), row);
        if hit.hovered { ui.c.fill_rrect(row, 7.0, poni_core::geom::Color::white(0.10)); }
        ui.c.icon(Icon::App, Rect::new(row.x + 8.0, row.y + 6.0, 18.0, 18.0), if hit.hovered { st.accent } else { palette::TEXT_DIM });
        ui.c.text(&app.name, Rect::new(row.x + 34.0, row.y, row.w - 42.0, row.h), &TextStyle::body().color(palette::TEXT));
        if hit.clicked { st.cmd(Command::OpenPath(app.path.clone())); }
    }
}
