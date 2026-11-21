//! "La tua settimana": what did good first, then the check-in trends (one small
//! chart per item, gaps where there is no answer), then simple associations.

use dioxus::prelude::*;
use jiff::civil::Date;
use kuracare_core::kinds::{CheckinItem, Route};
use kuracare_core::summary::{
    self, ASSOC_MIN_DAYS, ASSOC_WEEKS, Association, Counted, FactorKind, Trend,
};
use kuracare_core::time;

use crate::labels::{MONTHS, WEEKDAYS, checkin_label, route_label};
use crate::state::use_app;
use crate::views::parts::{Chip, Icon};

const DAY_INITIALS: [&str; 7] = ["L", "M", "M", "G", "V", "S", "D"];

/// One decimal, Italian comma: 6,5.
fn one(v: f64) -> String {
    format!("{v:.1}").replace('.', ",")
}

fn add_days(d: Date, n: i64) -> Date {
    d.checked_add(jiff::Span::new().days(n)).unwrap_or(d)
}

/// "16 – 22 novembre", or "30 novembre – 6 dicembre".
fn week_range(start: Date, end: Date) -> String {
    let m = |d: Date| MONTHS[(d.month() - 1) as usize];
    if start.month() == end.month() {
        format!("{} – {} {}", start.day(), end.day(), m(end))
    } else {
        format!("{} {} – {} {}", start.day(), m(start), end.day(), m(end))
    }
}

fn factor_label(a: &Association) -> String {
    match a.kind {
        FactorKind::Strain => format!("Giorni con {}", a.label),
        FactorKind::Route => {
            let r = match a.label.as_str() {
                "inhalation" => Route::Inhalation,
                "vaporization" => Route::Vaporization,
                "oral" => Route::Oral,
                _ => Route::Sublingual,
            };
            format!("Giorni con via {}", route_label(r).to_lowercase())
        }
        FactorKind::Care => format!("Giorni con «{}»", a.label),
        FactorKind::HomeCare => "Giorni in cui hai curato la casa".into(),
    }
}

#[component]
pub fn Summary() -> Element {
    let app = use_app();
    let mut offset = use_signal(|| 0i64);
    let mut assoc_item = use_signal(|| CheckinItem::MentalLoad);

    let tz = jiff::tz::TimeZone::system();
    let this_week = summary::week_start(time::now().date());
    let start = add_days(this_week, 7 * offset());
    let s = summary::weekly(&app.events.read(), start, &tz);
    let item_index = CheckinItem::ALL
        .iter()
        .position(|i| *i == assoc_item())
        .unwrap_or(0);
    let assoc: Vec<Association> = s
        .associations
        .iter()
        .filter(|a| a.with[item_index].is_some() && a.without[item_index].is_some())
        .cloned()
        .collect();

    rsx! {
        section { class: "fade summary",
            div { class: "week-nav",
                button {
                    class: "small", aria_label: "Settimana precedente",
                    onclick: move |_| offset -= 1,
                    "‹"
                }
                div { class: "week-title",
                    h1 { if offset() == 0 { "La tua settimana" } else { "Una settimana fa" } }
                    p { class: "muted", {week_range(s.start, s.end)} }
                }
                button {
                    class: "small", aria_label: "Settimana successiva",
                    disabled: offset() >= 0,
                    onclick: move |_| offset += 1,
                    "›"
                }
            }

            // 1. What did good.
            h2 { class: "sec", Icon { name: "care" } "Cosa ti ha fatto bene" }
            if !s.has_good_things() {
                p { class: "muted", "Questa settimana qui non c'è ancora niente. Va bene così." }
            }
            if !s.reflections.is_empty() {
                ul { class: "reflections",
                    for (i, (d, text)) in s.reflections.iter().enumerate() {
                        li { key: "{i}",
                            span { class: "time", {WEEKDAYS[d.weekday().to_monday_zero_offset() as usize]} }
                            blockquote { "{text}" }
                        }
                    }
                }
            }
            if !s.care.is_empty() {
                p { class: "label", "Gesti di cura" }
                Bars { rows: s.care.clone() }
            }
            if !s.home.is_empty() {
                p { class: "label", "La casa" }
                Bars { rows: s.home.clone() }
            }
            if !s.tasks_done.is_empty() {
                p { class: "label", "Cose portate a termine" }
                ul { class: "plain done-list",
                    for (i, t) in s.tasks_done.iter().enumerate() {
                        li { key: "{i}", span { class: "dot" } "{t}" }
                    }
                }
            }

            // 2. Trends.
            h2 { class: "sec", Icon { name: "checkin" } "Gli andamenti" }
            if s.checkin_days == 0 {
                p { class: "muted", "Nessun check-in in questa settimana: i grafici restano vuoti, e va bene così." }
            } else {
                p { class: "muted", "La media di ogni giorno, da 0 a 10. Tocca un punto per leggere il valore." }
            }
            for t in s.trends.iter().cloned() {
                TrendChart { key: "{t.item:?}", trend: t, start: s.start }
            }
            if s.checkin_days > 0 {
                details { class: "numbers",
                    summary { "Mostra i numeri" }
                    div { class: "table-wrap",
                        table {
                            thead { tr {
                                th { scope: "col", "" }
                                for k in 0..7 { th { key: "{k}", scope: "col", {DAY_INITIALS[k]} } }
                            } }
                            tbody {
                                for t in s.trends.iter() {
                                    tr { key: "{t.item:?}",
                                        th { scope: "row", {checkin_label(t.item).0} }
                                        for (k, v) in t.days.iter().enumerate() {
                                            td { key: "{k}", {v.map(one).unwrap_or_else(|| "–".into())} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Associations.
            h2 { class: "sec", Icon { name: "sparkle" } "Associazioni semplici" }
            p { class: "muted",
                "Le ultime {ASSOC_WEEKS} settimane: la media dei giorni con una certa cosa, accanto a quella dei giorni senza."
            }
            div { class: "chips", role: "group", aria_label: "Quale voce confrontare",
                for item in CheckinItem::ALL {
                    Chip {
                        key: "{item:?}",
                        label: checkin_label(item).0,
                        pressed: assoc_item() == item,
                        onclick: move |_| assoc_item.set(item),
                    }
                }
            }
            if assoc.is_empty() {
                p { class: "hint",
                    "Per confrontare servono almeno {ASSOC_MIN_DAYS} giorni con un check-in sia con sia senza quella cosa."
                }
            }
            for (i, a) in assoc.iter().enumerate() {
                div { key: "{i}", class: "assoc",
                    p { class: "assoc-title", {factor_label(a)} }
                    Compare {
                        label: format!("con ({} giorni)", a.days_with),
                        value: a.with[item_index].unwrap_or(0.0),
                        strong: true,
                    }
                    Compare {
                        label: format!("senza ({} giorni)", a.days_without),
                        value: a.without[item_index].unwrap_or(0.0),
                        strong: false,
                    }
                }
            }
            p { class: "note", "Un'associazione non indica una causa." }

            div { class: "actions",
                button { class: "quiet", onclick: move |_| app.back(), "Torna alla Home" }
            }
        }
    }
}

/// Horizontal bars: how many times, saffron (these are the good things).
#[component]
fn Bars(rows: Vec<Counted>) -> Element {
    let max = rows.iter().map(|r| r.times).max().unwrap_or(1).max(1);
    rsx! {
        ul { class: "bars",
            for r in rows {
                li { key: "{r.label}",
                    div { class: "bar-head",
                        span { "{r.label}" }
                        span { class: "muted",
                            if r.times == 1 { "1 volta" } else { "{r.times} volte" }
                            if let Some(f) = r.fulfilment { " · appagamento {one(f)}" }
                        }
                    }
                    div { class: "bar-track", aria_hidden: "true",
                        div { class: "bar-fill good", style: "width: {r.times as f64 / max as f64 * 100.0}%" }
                    }
                }
            }
        }
    }
}

/// One association side: a bar on the 0-10 scale with its value.
#[component]
fn Compare(label: String, value: f64, strong: bool) -> Element {
    rsx! {
        div { class: "compare",
            span { class: "compare-label", "{label}" }
            div { class: "bar-track", aria_hidden: "true",
                div {
                    class: if strong { "bar-fill" } else { "bar-fill faint" },
                    style: "width: {value / 10.0 * 100.0}%",
                }
            }
            span { class: "compare-val", {one(value)} }
        }
    }
}

// Chart geometry (SVG units).
const W: f64 = 320.0;
const H: f64 = 132.0;
const LEFT: f64 = 26.0;
const RIGHT: f64 = 10.0;
const TOP: f64 = 10.0;
const BOTTOM: f64 = 24.0;

fn x(k: usize) -> f64 {
    LEFT + (W - LEFT - RIGHT) * (k as f64 + 0.5) / 7.0
}

fn y(v: f64) -> f64 {
    TOP + (H - TOP - BOTTOM) * (10.0 - v.clamp(0.0, 10.0)) / 10.0
}

/// Line through consecutive days only: a missing day breaks the line.
fn line_path(days: &[Option<f64>; 7]) -> String {
    let mut d = String::new();
    let mut prev = false;
    for (k, v) in days.iter().enumerate() {
        match v {
            Some(v) => {
                d += &format!("{}{:.1} {:.1} ", if prev { "L" } else { "M" }, x(k), y(*v));
                prev = true;
            }
            None => prev = false,
        }
    }
    d
}

#[component]
fn TrendChart(trend: Trend, start: Date) -> Element {
    let mut picked = use_signal(|| None::<usize>);
    let (name, low, high) = checkin_label(trend.item);
    let caption = match picked() {
        Some(k) => {
            let day = add_days(start, k as i64);
            let wd = WEEKDAYS[day.weekday().to_monday_zero_offset() as usize];
            match trend.days[k] {
                Some(v) => format!("{wd} {}: {}", day.day(), one(v)),
                None => format!("{wd} {}: nessun check-in", day.day()),
            }
        }
        None => match (trend.mean, trend.previous_mean) {
            (Some(m), Some(p)) => format!("media {} · la settimana prima {}", one(m), one(p)),
            (Some(m), None) => format!("media {}", one(m)),
            _ => "nessuna risposta questa settimana".into(),
        },
    };
    let path = line_path(&trend.days);
    rsx! {
        div { class: "chart-card",
            div { class: "chart-head",
                span { class: "chart-title", "{name}" }
                span { class: "chart-caption", aria_live: "polite", "{caption}" }
            }
            svg {
                class: "chart",
                view_box: "0 0 {W} {H}",
                role: "img",
                "aria-label": "{name}: {caption}",
                // Gridlines at 0, 5, 10.
                for v in [0.0, 5.0, 10.0] {
                    g { key: "g{v}",
                        line { class: "grid", x1: "{LEFT}", x2: "{W - RIGHT}", y1: "{y(v)}", y2: "{y(v)}" }
                        text { class: "axis", x: "{LEFT - 8.0}", y: "{y(v) + 3.5}", text_anchor: "end", "{v}" }
                    }
                }
                if !path.is_empty() {
                    path { class: "line", d: "{path}" }
                }
                for k in 0..7 {
                    g { key: "d{k}",
                    text { class: "axis", x: "{x(k)}", y: "{H - 6.0}", text_anchor: "middle", {DAY_INITIALS[k]} }
                    if let Some(v) = trend.days[k] {
                        circle {
                            class: if picked() == Some(k) { "point picked" } else { "point" },
                            cx: "{x(k)}", cy: "{y(v)}", r: if picked() == Some(k) { "6" } else { "4.5" },
                        }
                    }
                    // A tall, invisible target per day: easy to tap.
                    rect {
                        class: "hit",
                        x: "{x(k) - 19.0}", y: "0", width: "38", height: "{H}",
                        onclick: move |_| picked.set(if picked() == Some(k) { None } else { Some(k) }),
                    }
                    }
                }
            }
            div { class: "scale-ends", aria_hidden: "true",
                span { "0 · {low}" }
                span { "{high} · 10" }
            }
        }
    }
}
