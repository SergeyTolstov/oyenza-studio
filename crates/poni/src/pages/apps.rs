//! Windows Start-menu apps. The list is fetched once when this page opens and
//! filtered locally; it neither watches installed software nor runs at rest.

use crate::commands::Command;
use crate::gfx::TextStyle;
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, wid_n, PillStyle, Ui, PILL_SMALL};
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 240.0;
const SEARCH_H: f32 = 34.0;
const FILTER_Y: f32 = SEARCH_H + 8.0;
const LIST_Y: f32 = FILTER_Y + PILL_SMALL + 8.0;
const ROW_H: f32 = 31.0;
const MAX_ROWS: usize = 5;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let search = Rect::new(area.x, area.y, area.w, SEARCH_H);
    ui.text_input_row(wid("apps.search"), search, &mut st.ui.apps_query, "Search installed apps", 80, st.accent, Some(Icon::App));
    let fav_label = format!("Favorites ({})", st.store.apps_favorites.len());
    let fav_r = Rect::new(area.x, area.y + FILTER_Y, 136.0, PILL_SMALL);
    let recent_r = Rect::new(fav_r.right() + 8.0, fav_r.y, 94.0, PILL_SMALL);
    let all_r = Rect::new(recent_r.right() + 8.0, fav_r.y, 66.0, PILL_SMALL);
    let refresh_r = Rect::new(all_r.right() + 8.0, fav_r.y, PILL_SMALL, PILL_SMALL);
    if ui.pill(wid("apps.favorites"), fav_r, &fav_label, Some(Icon::PinFill), if st.ui.apps_favorites_only { PillStyle::Primary(st.accent) } else { PillStyle::Neutral }).clicked {
        st.ui.apps_favorites_only = true;
        st.ui.apps_recent_only = false;
    }
    if ui.pill(wid("apps.recent"), recent_r, "Recent", Some(Icon::Clock), if st.ui.apps_recent_only { PillStyle::Primary(st.accent) } else { PillStyle::Neutral }).clicked {
        st.ui.apps_favorites_only = false;
        st.ui.apps_recent_only = true;
    }
    if ui.pill(wid("apps.all"), all_r, "All", Some(Icon::App), if !st.ui.apps_favorites_only && !st.ui.apps_recent_only { PillStyle::Primary(st.accent) } else { PillStyle::Neutral }).clicked {
        st.ui.apps_favorites_only = false;
        st.ui.apps_recent_only = false;
    }
    if ui.icon_button(wid("apps.refresh"), refresh_r, Icon::Reset, palette::TEXT_DIM, 14.0).clicked && !st.ui.apps_loading {
        st.ui.apps_loading = true;
        st.cmd(Command::RefreshApps);
    }
    if st.launch_apps.is_empty() {
        super::empty_state(ui, Rect::new(area.x, area.y + LIST_Y, area.w, HEIGHT - LIST_Y), Icon::App, "Loading apps", "Open this page to refresh your Start menu");
        if !st.ui.apps_loading {
            st.ui.apps_loading = true;
            st.cmd(Command::RefreshApps);
        }
        return;
    }
    let words: Vec<String> = st.ui.apps_query.split_whitespace().map(|s| s.to_lowercase()).collect();
    let source: Vec<_> = if st.ui.apps_favorites_only {
        st.store.apps_favorites.iter().filter_map(|path| st.launch_apps.iter().find(|a| &a.path == path)).collect()
    } else if st.ui.apps_recent_only {
        st.store.apps_recent.iter().filter_map(|path| st.launch_apps.iter().find(|a| &a.path == path)).collect()
    } else { st.launch_apps.iter().collect() };
    let matches: Vec<_> = source.into_iter().filter(|a| {
        let name = a.name.to_lowercase();
        words.iter().all(|w| name.contains(w))
    }).take(MAX_ROWS).cloned().collect();
    if matches.is_empty() {
        let (title, hint) = if st.ui.apps_favorites_only { ("No favorites yet", "Pin an app from All to keep it here") } else if st.ui.apps_recent_only { ("No recent apps", "Launch an app and it will appear here") } else { ("No matching apps", "Try a different name") };
        super::empty_state(ui, Rect::new(area.x, area.y + LIST_Y, area.w, HEIGHT - LIST_Y), Icon::App, title, hint);
        return;
    }
    for (i, app) in matches.iter().enumerate() {
        let row = Rect::new(area.x, area.y + LIST_Y + i as f32 * ROW_H, area.w, ROW_H - 2.0);
        let launch = Rect::new(row.x, row.y, row.w - 34.0, row.h);
        let hit = ui.button(wid_n("apps.launch", i as u64), launch);
        if hit.hovered { ui.c.fill_rrect(row, 7.0, poni_core::geom::Color::white(0.10)); }
        ui.c.icon(Icon::App, Rect::new(row.x + 8.0, row.y + 6.0, 18.0, 18.0), if hit.hovered { st.accent } else { palette::TEXT_DIM });
        ui.c.text(&app.name, Rect::new(row.x + 34.0, row.y, row.w - 70.0, row.h), &TextStyle::body().color(palette::TEXT));
        let saved = st.store.apps_favorites.iter().any(|p| p == &app.path);
        let pin = Rect::new(row.right() - 28.0, row.y + 3.0, 25.0, 25.0);
        if ui.icon_button(wid_n("apps.pin", i as u64), pin, if saved { Icon::PinFill } else { Icon::Pin }, if saved { st.accent } else { palette::TEXT_DIM }, 14.0).clicked {
            if saved { st.store.apps_favorites.retain(|p| p != &app.path); } else { st.store.apps_favorites.push(app.path.clone()); }
            st.save();
        }
        if hit.clicked {
            st.store.apps_recent.retain(|p| p != &app.path);
            st.store.apps_recent.insert(0, app.path.clone());
            st.store.apps_recent.truncate(12);
            st.save();
            st.cmd(Command::OpenPath(app.path.clone()));
        }
    }
}
