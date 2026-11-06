//! A local, compact summary of the current day at the computer.

use crate::gfx::{TextStyle, Weight};
use crate::icons::Icon;
use crate::pages::tr;
use crate::state::AppState;
use crate::ui::Ui;
use poni_core::datetime::fmt_duration;
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 360.0;
const ROW_H: f32 = 27.0;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

pub fn next_wake(st: &AppState) -> Option<f64> {
    Some(st.now + (60 - st.local.second.min(59)) as f64)
}

fn fit_text(ui: &mut Ui, text: &str, style: &TextStyle, max_width: f32) -> String {
    if ui.c.measure(text, style, max_width).0 <= max_width { return text.to_string(); }
    let mut out = String::new();
    for ch in text.chars() {
        out.push(ch);
        if ui.c.measure(&(out.clone() + "…"), style, max_width).0 > max_width {
            out.pop();
            break;
        }
    }
    format!("{out}…")
}

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let tracker = &st.store.time_tracker;
    let day = st.local.day_key();
    let completed = (tracker.history_day == day).then_some(tracker.history.as_slice()).unwrap_or(&[]);
    let current_total: u64 = tracker.timers.iter().map(|timer| timer.seconds(st.unix)).sum();
    let completed_total: u64 = completed.iter().map(|timer| timer.elapsed).sum();
    let work_total = current_total.saturating_add(completed_total);
    let pending: Vec<_> = tracker.timers.iter().filter(|timer| {
        timer.planned_unix.is_some_and(|at| at >= st.day_start_unix() && at < st.day_start_unix() + 86_400)
    }).collect();

    ui.c.icon(Icon::Calendar, Rect::new(area.x, area.y + 2.0, 20.0, 20.0), st.accent);
    ui.c.text(tr(st, "Today on computer", "Сегодня за компьютером", "Бүгін компьютерде"), Rect::new(area.x + 28.0, area.y, area.w - 28.0, 22.0), &TextStyle::headline().weight(Weight::Semibold));
    let summary = match st.settings.language {
        poni_core::settings::Language::English => format!("{} completed · {} planned", completed.len(), pending.len()),
        poni_core::settings::Language::Russian => format!("{} завершено · {} запланировано", completed.len(), pending.len()),
        poni_core::settings::Language::Kazakh => format!("{} аяқталды · {} жоспарланды", completed.len(), pending.len()),
    };
    ui.c.text(&summary, Rect::new(area.x + 28.0, area.y + 21.0, area.w - 28.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

    let card = Rect::new(area.x, area.y + 48.0, area.w, 54.0);
    ui.c.fill_rrect(card, 13.0, palette::SURFACE);
    ui.c.icon(Icon::Timer, Rect::new(card.x + 14.0, card.y + 16.0, 20.0, 20.0), st.accent);
    ui.c.text(tr(st, "Tracked work", "Учтённая работа", "Есептелген жұмыс"), Rect::new(card.x + 44.0, card.y + 8.0, 150.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));
    ui.c.text(&fmt_duration(work_total as f64), Rect::new(card.x + 44.0, card.y + 23.0, card.w - 58.0, 23.0), &TextStyle::headline().weight(Weight::Semibold).tabular().color(st.accent));

    let mut y = area.y + 116.0;
    ui.c.text(tr(st, "Planned for today", "Запланировано на сегодня", "Бүгінге жоспарланған"), Rect::new(area.x, y, area.w, 18.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    y += 21.0;
    if pending.is_empty() {
        ui.c.text(tr(st, "Nothing pending for today", "На сегодня ничего не осталось", "Бүгінге ештеңе қалмады"), Rect::new(area.x + 4.0, y, area.w, ROW_H), &TextStyle::body().color(palette::TEXT_DIM));
        y += ROW_H;
    } else {
        for timer in pending.into_iter().take(3) {
            ui.c.icon(Icon::Calendar, Rect::new(area.x + 3.0, y + 6.0, 15.0, 15.0), palette::ORANGE);
            let style = TextStyle::body();
            let shown = fit_text(ui, &timer.comment, &style, area.w - 110.0);
            ui.c.text(&shown, Rect::new(area.x + 26.0, y, area.w - 112.0, ROW_H), &style);
            let at = timer.planned_unix.unwrap_or_default() - st.day_start_unix();
            ui.c.text(&format!("{:02}:{:02}", at / 3600, (at / 60) % 60), Rect::new(area.right() - 62.0, y, 62.0, ROW_H), &TextStyle::footnote().color(palette::TEXT_DIM).right().tabular());
            y += ROW_H;
        }
    }

    y += 7.0;
    ui.c.text(tr(st, "Most active apps", "Самые активные приложения", "Ең белсенді қолданбалар"), Rect::new(area.x, y, area.w, 18.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    y += 21.0;
    let mut apps: Vec<_> = if tracker.usage_day == day { tracker.apps.iter().collect() } else { Vec::new() };
    apps.sort_by_key(|(_, seconds)| std::cmp::Reverse(**seconds));
    if apps.is_empty() {
        ui.c.text(tr(st, "Activity will appear after the first minute", "Активность появится через минуту", "Белсенділік бір минуттан кейін көрінеді"), Rect::new(area.x + 4.0, y, area.w, ROW_H), &TextStyle::body().color(palette::TEXT_DIM));
    } else {
        for (name, seconds) in apps.into_iter().take(3) {
            ui.c.icon(Icon::App, Rect::new(area.x + 3.0, y + 6.0, 15.0, 15.0), st.accent);
            let style = TextStyle::body();
            let shown = fit_text(ui, name, &style, area.w - 112.0);
            ui.c.text(&shown, Rect::new(area.x + 26.0, y, area.w - 112.0, ROW_H), &style);
            ui.c.text(&fmt_duration(*seconds as f64), Rect::new(area.right() - 80.0, y, 80.0, ROW_H), &TextStyle::footnote().color(st.accent).right().tabular());
            y += ROW_H;
        }
    }
}
