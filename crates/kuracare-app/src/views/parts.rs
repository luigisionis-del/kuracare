use dioxus::prelude::*;
use kuracare_core::kinds::score;

use crate::state::{View, use_app};

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
                span { class: "scale-val", aria_hidden: "true", "{shown}" }
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
            div { class: "scale-ends", aria_hidden: "true",
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
