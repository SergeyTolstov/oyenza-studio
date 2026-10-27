//! Today is intentionally local: a quiet date card with no calendar account,
//! network request, meeting data, or Google sign-in.

use crate::gfx::{TextStyle, Weight};
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::Ui;
use poni_core::datetime::{fmt_clock, MONTH_SHORT, WEEKDAY_SHORT};
use poni_core::geom::{palette, Color, Rect};

const HEIGHT: f32 = 164.0;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

pub fn next_wake(st: &AppState) -> Option<f64> {
    Some(st.now + (60 - st.local.second.min(59)) as f64)
}

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let area = Rect::new(area.x, area.y, area.w, HEIGHT);
    let t = st.local;
    ui.c.fill_rrect(area, 18.0, palette::SURFACE);
    ui.c.icon(Icon::Calendar, Rect::new(area.x + 22.0, area.y + 24.0, 28.0, 28.0), st.accent);
    ui.c.text(&WEEKDAY_SHORT[t.weekday() as usize], Rect::new(area.x + 64.0, area.y + 20.0, area.w - 86.0, 22.0), &TextStyle::headline().color(palette::TEXT_DIM));
    ui.c.text(&t.day.to_string(), Rect::new(area.x + 20.0, area.y + 54.0, 118.0, 68.0), &TextStyle::hero().weight(Weight::Semibold).color(palette::TEXT));
    let month = format!("{} {}", MONTH_SHORT[(t.month.clamp(1, 12) - 1) as usize], t.year);
    ui.c.text(&month, Rect::new(area.x + 144.0, area.y + 65.0, area.w - 166.0, 24.0), &TextStyle::headline());
    ui.c.text(&fmt_clock(&t, st.settings.use_24h), Rect::new(area.x + 144.0, area.y + 93.0, area.w - 166.0, 22.0), &TextStyle::body().weight(Weight::Semibold).tabular().color(st.accent));
    ui.c.text("Today stays on this PC. No calendar account is connected.", Rect::new(area.x + 22.0, area.bottom() - 30.0, area.w - 44.0, 18.0), &TextStyle::footnote().color(Color::white(0.50)));
}
