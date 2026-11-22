use dioxus::prelude::*;
use kuracare_core::kinds::score;

use kuracare_core::Payload;

use crate::state::{View, use_app};

/// The icon of a kind of record.
pub fn kind_icon(p: &Payload) -> &'static str {
    match p {
        Payload::Checkin(_) | Payload::Questionnaire(_) => "checkin",
        Payload::Thought(_) => "thought",
        Payload::Care(_) => "care",
        Payload::Home(_) => "home",
        Payload::Reflection(_) => "evening",
        Payload::Cannabis(_) | Payload::Supplement(_) => "leaf",
        Payload::Meal(_) => "meal",
        Payload::Task(_) => "tasks",
        Payload::DataRelease(_) => "lock",
        Payload::GrowthRead(_) | Payload::GrowthPractice(_) | Payload::GrowthStage(_) => "sparkle",
    }
}

/// Top of every inner screen: back to Home, then the title with its icon.
#[component]
pub fn ScreenHead(title: String, icon: Option<&'static str>, subtitle: Option<String>) -> Element {
    let app = use_app();
    rsx! {
        header { class: "screen-head",
            button {
                class: "back",
                r#type: "button",
                aria_label: "Torna alla Home",
                onclick: move |_| app.back(),
                Icon { name: "back" }
                "Home"
            }
            h1 { class: "sec",
                if let Some(i) = icon { Icon { name: i } }
                "{title}"
            }
            if let Some(s) = subtitle { p { class: "muted", "{s}" } }
        }
    }
}

/// 0-10 answer that starts unanswered and can always be cleared.
#[component]
pub fn Scale(
    id: String,
    label: String,
    low: String,
    high: String,
    value: Option<u8>,
    onchange: EventHandler<Option<u8>>,
) -> Element {
    let shown = value.map(|v| v.to_string()).unwrap_or_else(|| "–".into());
    let valuetext = value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "non indicato".into());
    rsx! {
        div { class: "scale",
            div { class: "scale-head",
                label { r#for: "{id}", "{label}" }
                span { class: "scale-val", "aria-hidden": "true", "{shown}" }
                if value.is_some() {
                    button {
                        r#type: "button",
                        class: "quiet small",
                        aria_label: "Togli la risposta: {label}",
                        onclick: move |_| onchange.call(None),
                        "togli"
                    }
                }
            }
            input {
                id: "{id}",
                r#type: "range",
                min: "0",
                max: "10",
                step: "1",
                class: if value.is_none() { "unset" } else { "" },
                value: "{value.unwrap_or(5)}",
                aria_valuetext: "{valuetext}",
                oninput: move |e| onchange.call(score(&e.value())),
            }
            div { class: "scale-ticks", "aria-hidden": "true",
                for i in 0..=10 { span { key: "{i}" } }
            }
            div { class: "scale-ends", "aria-hidden": "true",
                span { "0 · {low}" }
                span { "{high} · 10" }
            }
        }
    }
}

/// Toggle chip; `pressed` shows the current choice.
#[component]
pub fn Chip(label: String, pressed: Option<bool>, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "chip",
            aria_pressed: pressed.map(|p| p.to_string()),
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

#[component]
pub fn ToastBar() -> Element {
    let app = use_app();
    let toast = app.toast.read().clone();
    rsx! {
        if let Some(t) = toast {
            div { class: "toast fade", role: "status",
                span { "{t.text}" }
                if let Some((label, id)) = t.action {
                    button {
                        class: "small",
                        onclick: move |_| app.go(View::Edit(id.clone())),
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Small line icons drawn for Kuracare (24 x 24, stroked with the theme's icon colour).
fn icon_paths(name: &str) -> &'static [&'static str] {
    match name {
        // A calm face: "Come stai?"
        "checkin" => &[
            "M20.5 12a8.5 8.5 0 1 1-17 0 8.5 8.5 0 0 1 17 0z",
            "M8.6 14.3c1.9 1.9 4.9 1.9 6.8 0",
            "M9.2 9.8h.01M14.8 9.8h.01",
        ],
        "tasks" => &[
            "M10 6.5h10M10 12h10M10 17.5h10",
            "M3.6 6.7l1.3 1.3 2.3-2.6M3.6 12.2l1.3 1.3 2.3-2.6M3.6 17.7l1.3 1.3 2.3-2.6",
        ],
        "care" => &["M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"],
        "home" => &["M4 11l8-6 8 6", "M6 10v9.5h12V10", "M10 19.5v-5h4v5"],
        "leaf" => &[
            "M5 19c0-8.5 5-13.2 14-14 0 9-5 14-13.2 14z",
            "M5 19l7.5-7.5",
        ],
        "meal" => &[
            "M3.5 11.5h17a8.5 8.5 0 0 1-17 0z",
            "M9.5 8c0-1.4 1-1.9 1-3.3M13.5 8c0-1.4 1-1.9 1-3.3",
        ],
        "thought" => &["M7.2 18h9.6a4 4 0 0 0 .6-7.95A5.5 5.5 0 0 0 6.9 9.6 4.2 4.2 0 0 0 7.2 18z"],
        "evening" => &["M19 14.6A7.6 7.6 0 0 1 9.4 5a7.6 7.6 0 1 0 9.6 9.6z"],
        "sparkle" => &["M12 3.5l1.9 5.6 5.6 1.9-5.6 1.9-1.9 5.6-1.9-5.6-5.6-1.9 5.6-1.9z"],
        "today" => &[
            "M20.5 12a8.5 8.5 0 1 1-17 0 8.5 8.5 0 0 1 17 0z",
            "M12 7.5V12l3 2",
        ],
        "check" => &["M5 12.5l4.2 4.2L19 7"],
        "palette" => &[
            "M12 3.5a8.5 8.5 0 1 0 0 17c1.2 0 1.8-.8 1.8-1.7 0-1.2-1-1.6-1-2.6 0-.9.7-1.5 1.6-1.5h2.1c2.2 0 4-1.8 4-4 0-4-3.8-7.2-8.5-7.2z",
            "M7.8 11.5h.01M10 7.8h.01M14.3 7.8h.01",
        ],
        "bell" => &[
            "M6 16.5V11a6 6 0 0 1 12 0v5.5l1.5 1.5h-15z",
            "M10 20.5a2 2 0 0 0 4 0",
        ],
        "pause" => &["M9 6v12M15 6v12"],
        "lock" => &["M6.5 11h11v9h-11z", "M8.5 11V8a3.5 3.5 0 0 1 7 0v3"],
        "list" => &["M5 7h14M5 12h14M5 17h9"],
        "back" => &["M14.5 5.5L8 12l6.5 6.5"],
        "eye" => &[
            "M2.5 12s3.5-6.5 9.5-6.5 9.5 6.5 9.5 6.5-3.5 6.5-9.5 6.5S2.5 12 2.5 12z",
            "M14.8 12a2.8 2.8 0 1 1-5.6 0 2.8 2.8 0 0 1 5.6 0z",
        ],
        "close" => &["M7.5 7.5l9 9M16.5 7.5l-9 9"],
        _ => &[],
    }
}

#[component]
pub fn Icon(name: &'static str) -> Element {
    rsx! {
        svg {
            class: "icon",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            for (i, d) in icon_paths(name).iter().enumerate() {
                path { key: "{i}", d: *d }
            }
        }
    }
}

/// The Kuracare emblem (same drawing as the app icon): a heart with three
/// saffron stigmas, like a crocus flower.
#[component]
pub fn Emblem() -> Element {
    rsx! {
        svg { class: "emblem", view_box: "0 0 72 72", "aria-hidden": "true",
            circle { cx: "36", cy: "36", r: "34", fill: "var(--accent-soft)" }
            g { transform: "translate(36 37.5) scale(0.15) translate(-256 -262)",
                path {
                    d: "M256 392c-74-46-124-96-124-156 0-40 30-70 66-70 26 0 46 13 58 33 12-20 32-33 58-33 36 0 66 30 66 70 0 60-50 110-124 156z",
                    fill: "var(--accent)",
                }
                g { fill: "none", stroke: "var(--bloom)", stroke_width: "15", stroke_linecap: "round",
                    path { d: "M256 318C248 262 222 206 192 166" }
                    path { d: "M256 318C256 254 256 196 256 140" }
                    path { d: "M256 318C264 262 290 206 320 166" }
                }
                g { fill: "var(--bloom)",
                    ellipse { cx: "188", cy: "158", rx: "16", ry: "21", transform: "rotate(-34 188 158)" }
                    ellipse { cx: "256", cy: "130", rx: "16", ry: "21" }
                    ellipse { cx: "324", cy: "158", rx: "16", ry: "21", transform: "rotate(34 324 158)" }
                }
            }
        }
    }
}
