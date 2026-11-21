use dioxus::prelude::*;
use kuracare_core::kinds::{CheckinItem, MealType, Payload, Route, positive};
use kuracare_core::{Event, repo, time};

use crate::labels::{checkin_label, describe, meal_label, num, route_label};
use crate::photo;
use crate::platform::vault;
use crate::state::use_app;
use crate::views::parts::{Chip, Scale};

const DURATIONS: [u32; 5] = [5, 10, 20, 30, 60];

fn int(n: Option<u32>) -> String {
    n.map(|n| n.to_string()).unwrap_or_default()
}

#[component]
pub fn Edit(id: String) -> Element {
    let app = use_app();
    let original = use_hook(|| app.find(&id));
    let Some(original) = original else {
        return rsx! {
            p { class: "muted", "Questa registrazione non c'è più." }
            button { onclick: move |_| app.back(), "Torna indietro" }
        };
    };

    let mut draft = use_signal(|| original.clone());
    let mut when = use_signal(|| time::to_local_input(&original.ts));
    let (a0, t0, c0, d0) = match &original.payload {
        Payload::Cannabis(c) => (num(c.amount), num(c.thc_pct), num(c.cbd_pct), String::new()),
        Payload::Supplement(s) => (num(s.amount), String::new(), String::new(), String::new()),
        Payload::Care(c) => (
            String::new(),
            String::new(),
            String::new(),
            int(c.duration_min),
        ),
        Payload::Home(h) => (
            String::new(),
            String::new(),
            String::new(),
            int(h.duration_min),
        ),
        _ => Default::default(),
    };
    let mut amount = use_signal(|| a0);
    let mut thc = use_signal(|| t0);
    let mut cbd = use_signal(|| c0);
    let duration = use_signal(|| d0);
    let photo_url = use_signal(|| match &original.payload {
        Payload::Meal(m) => m
            .photo_id
            .as_deref()
            .and_then(|id| repo::load_photo(&vault(), id).ok().flatten())
            .map(|b| photo::data_url(&b)),
        _ => None,
    });
    let mut confirm_delete = use_signal(|| false);

    let save = move |_| {
        let mut e: Event = draft.read().clone();
        if let Some(ts) = time::from_local_input(&when.read()) {
            e.ts = ts;
        }
        match &mut e.payload {
            Payload::Cannabis(c) => {
                c.amount = positive(&amount.read(), f64::INFINITY);
                c.thc_pct = positive(&thc.read(), 100.0);
                c.cbd_pct = positive(&cbd.read(), 100.0);
                c.strain = c.strain.trim().to_owned();
                if c.strain.is_empty() {
                    c.strain = "Senza nome".into();
                }
            }
            Payload::Supplement(s) => s.amount = positive(&amount.read(), f64::INFINITY),
            Payload::Care(c) => {
                c.duration_min = positive(&duration.read(), 1440.0).map(|m| m.round() as u32)
            }
            Payload::Home(h) => {
                h.duration_min = positive(&duration.read(), 1440.0).map(|m| m.round() as u32)
            }
            Payload::Meal(m) => {
                m.description = m
                    .description
                    .take()
                    .map(|d| d.trim().to_owned())
                    .filter(|d| !d.is_empty())
            }
            Payload::Thought(t) => {
                t.note = t
                    .note
                    .take()
                    .map(|d| d.trim().to_owned())
                    .filter(|d| !d.is_empty())
            }
            _ => {}
        }
        app.change(&e);
        app.back();
    };

    let remove = move |_| {
        app.forget(&draft.read());
        app.back();
    };

    let d = draft.read().clone();
    let lists = app.lists.read().clone();

    rsx! {
        section { class: "fade",
            h1 { {describe(&d)} }
            label { r#for: "when", "Quando" }
            input { id: "when", r#type: "datetime-local", value: "{when}", oninput: move |e| when.set(e.value()) }

            match &d.payload {
                Payload::Checkin(c) => rsx! {
                    p { class: "hint", "Le risposte del check-in restano com'erano in quel momento: puoi cambiarne solo l'ora." }
                    ul { class: "plain",
                        for item in CheckinItem::ALL {
                            li { key: "{item:?}", "{checkin_label(item).0}: " {item.get(c).map(|v| v.to_string()).unwrap_or_else(|| "–".into())} }
                        }
                    }
                },
                Payload::Care(c) => rsx! {
                    Scale {
                        id: "ful", label: "Quanto ti ha fatto bene?", low: "poco", high: "moltissimo", value: c.fulfilment,
                        onchange: move |v| if let Payload::Care(c) = &mut draft.write().payload { c.fulfilment = v },
                    }
                    Duration { value: duration }
                },
                Payload::Home(h) => rsx! {
                    p { class: "label", "Cosa hai fatto" }
                    div { class: "chips", role: "group", aria_label: "Cosa hai fatto",
                        for a in lists.home_activities.iter().cloned() {
                            Chip {
                                key: "{a}",
                                label: a.clone(),
                                pressed: h.activity.as_deref() == Some(a.as_str()),
                                onclick: move |_| if let Payload::Home(h) = &mut draft.write().payload {
                                    h.activity = if h.activity.as_deref() == Some(a.as_str()) { None } else { Some(a.clone()) };
                                },
                            }
                        }
                    }
                    p { class: "label", "Stanza" }
                    div { class: "chips", role: "group", aria_label: "Stanza",
                        for r in lists.rooms.iter().cloned() {
                            Chip {
                                key: "{r}",
                                label: r.clone(),
                                pressed: h.room == r,
                                onclick: move |_| if let Payload::Home(h) = &mut draft.write().payload { h.room = r.clone() },
                            }
                        }
                    }
                    Scale {
                        id: "hful", label: "Quanto ti ha fatto bene?", low: "poco", high: "moltissimo", value: h.fulfilment,
                        onchange: move |v| if let Payload::Home(h) = &mut draft.write().payload { h.fulfilment = v },
                    }
                    Duration { value: duration }
                },
                Payload::Cannabis(c) => rsx! {
                    label { r#for: "st", "Varietà" }
                    input {
                        id: "st", r#type: "text", value: "{c.strain}",
                        oninput: move |e| if let Payload::Cannabis(c) = &mut draft.write().payload { c.strain = e.value() },
                    }
                    div { class: "grid2",
                        div { label { r#for: "thc", "THC %" } input { id: "thc", r#type: "text", inputmode: "decimal", value: "{thc}", oninput: move |e| thc.set(e.value()) } }
                        div { label { r#for: "cbd", "CBD %" } input { id: "cbd", r#type: "text", inputmode: "decimal", value: "{cbd}", oninput: move |e| cbd.set(e.value()) } }
                        div { label { r#for: "am", "Quantità" } input { id: "am", r#type: "text", inputmode: "decimal", value: "{amount}", oninput: move |e| amount.set(e.value()) } }
                        div {
                            label { r#for: "un", "Unità" }
                            input {
                                id: "un", r#type: "text", value: "{c.unit}",
                                oninput: move |e| if let Payload::Cannabis(c) = &mut draft.write().payload { c.unit = e.value() },
                            }
                        }
                    }
                    p { class: "label", "Via di assunzione" }
                    div { class: "chips", role: "group", aria_label: "Via di assunzione",
                        for r in Route::ALL {
                            Chip {
                                key: "{r:?}",
                                label: route_label(r),
                                pressed: c.route == Some(r),
                                onclick: move |_| if let Payload::Cannabis(c) = &mut draft.write().payload {
                                    c.route = if c.route == Some(r) { None } else { Some(r) };
                                },
                            }
                        }
                    }
                },
                Payload::Supplement(s) => rsx! {
                    div { class: "grid2",
                        div { label { r#for: "am", "Quantità" } input { id: "am", r#type: "text", inputmode: "decimal", value: "{amount}", oninput: move |e| amount.set(e.value()) } }
                        div {
                            label { r#for: "un", "Unità" }
                            input {
                                id: "un", r#type: "text", value: "{s.unit}",
                                oninput: move |e| if let Payload::Supplement(s) = &mut draft.write().payload { s.unit = e.value() },
                            }
                        }
                    }
                },
                Payload::Meal(m) => rsx! {
                    div { class: "chips", role: "group", aria_label: "Quale pasto",
                        for t in MealType::ALL {
                            Chip {
                                key: "{t:?}", label: meal_label(t), pressed: m.meal_type == t,
                                onclick: move |_| if let Payload::Meal(m) = &mut draft.write().payload { m.meal_type = t },
                            }
                        }
                    }
                    label { r#for: "md", "Cosa c'era" }
                    input {
                        id: "md", r#type: "text", maxlength: "140", value: m.description.clone().unwrap_or_default(),
                        oninput: move |e| if let Payload::Meal(m) = &mut draft.write().payload { m.description = Some(e.value()) },
                    }
                    // Photos taken before they were paused are still shown.
                    if let Some(url) = photo_url() { img { class: "photo", src: url, alt: "Foto del pasto" } }
                },
                Payload::Thought(t) => rsx! {
                    div { class: "chips", role: "group", aria_label: "Che tipo di pensiero",
                        for l in lists.thought_labels.iter().cloned() {
                            Chip {
                                key: "{l}",
                                label: l.clone(),
                                pressed: t.label.as_deref() == Some(l.as_str()),
                                onclick: move |_| if let Payload::Thought(t) = &mut draft.write().payload {
                                    t.label = if t.label.as_deref() == Some(l.as_str()) { None } else { Some(l.clone()) };
                                },
                            }
                        }
                    }
                    label { r#for: "tn", "Due parole " span { class: "muted", "(restano solo qui)" } }
                    input {
                        id: "tn", r#type: "text", maxlength: "280", value: t.note.clone().unwrap_or_default(),
                        oninput: move |e| if let Payload::Thought(t) = &mut draft.write().payload { t.note = Some(e.value()) },
                    }
                },
                Payload::Reflection(r) => rsx! {
                    label { r#for: "rf", "Cosa ti ha fatto bene" }
                    input {
                        id: "rf", r#type: "text", maxlength: "200", value: "{r.text}",
                        oninput: move |e| if let Payload::Reflection(r) = &mut draft.write().payload { r.text = e.value() },
                    }
                },
            }

            div { class: "actions",
                button { class: "primary", onclick: save, "Salva" }
                button { class: "quiet", onclick: move |_| app.back(), "Lascia com'era" }
                if !confirm_delete() {
                    button { class: "quiet", onclick: move |_| confirm_delete.set(true), "Elimina questa registrazione" }
                } else {
                    button { class: "danger", onclick: remove, "Sì, eliminala" }
                }
            }
        }
    }
}

/// Minutes, with quick choices.
#[component]
fn Duration(value: Signal<String>) -> Element {
    let mut value = value;
    rsx! {
        label { r#for: "dur", "Quanto è durato " span { class: "muted", "(minuti)" } }
        div { class: "chips",
            for m in DURATIONS {
                Chip {
                    key: "{m}",
                    label: m.to_string(),
                    pressed: *value.read() == m.to_string(),
                    onclick: move |_| {
                        let same = *value.read() == m.to_string();
                        value.set(if same { String::new() } else { m.to_string() });
                    },
                }
            }
        }
        input { id: "dur", r#type: "text", inputmode: "numeric", value: "{value}", oninput: move |e| value.set(e.value()) }
    }
}
