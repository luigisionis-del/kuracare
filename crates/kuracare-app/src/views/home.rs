use dioxus::prelude::*;
use kuracare_core::kinds::{
    Cannabis, Care, Home as HomeCare, MealType, Payload, Reflection, Supplement,
};
use kuracare_core::lists::{StrainGroup, StrainPreset, SupplementPreset};
use kuracare_core::time;

use crate::labels::{describe, greeting, long_date, meal_label};
use crate::state::{View, use_app};

#[component]
pub fn Home() -> Element {
    let app = use_app();
    let mut reflection_text = use_signal(String::new);

    let now = time::now();
    let today = time::today();
    let evening = now.hour() >= 18;
    let settings = app.settings.read().clone();
    let paused = settings
        .pause_until
        .as_deref()
        .and_then(|p| time::parse(p).ok())
        .is_some_and(|p| p > now.timestamp());
    let lists = app.lists.read().clone();
    let todays: Vec<_> = app
        .events
        .read()
        .iter()
        .filter(|e| time::local_day(&e.ts) == today)
        .cloned()
        .collect();
    let reflection = todays
        .iter()
        .find(|e| matches!(e.payload, Payload::Reflection(_)))
        .cloned();
    let meals_left: Vec<MealType> = [MealType::Breakfast, MealType::Lunch, MealType::Dinner]
        .into_iter()
        .filter(|t| {
            !todays
                .iter()
                .any(|e| matches!(&e.payload, Payload::Meal(m) if m.meal_type == *t))
        })
        .collect();
    let show_meal_recall = evening
        && !paused
        && !meals_left.is_empty()
        && settings.meal_recall_dismissed_on.as_deref() != Some(today.as_str());
    let thc: Vec<StrainPreset> = lists
        .strains
        .iter()
        .filter(|s| s.group == StrainGroup::Thc)
        .cloned()
        .collect();
    let cbd: Vec<StrainPreset> = lists
        .strains
        .iter()
        .filter(|s| s.group == StrainGroup::Cbd)
        .cloned()
        .collect();

    let care = move |label: String| {
        if let Some(e) = app.record(Payload::Care(Care {
            label,
            duration_min: None,
            fulfilment: None,
        })) {
            app.show_toast(
                app.recognition(),
                Some(("Quanto ti ha fatto bene?".into(), e.id)),
            );
        }
    };
    let room = move |room: String| {
        if let Some(e) = app.record(Payload::Home(HomeCare {
            room,
            activity: None,
            duration_min: None,
            fulfilment: None,
        })) {
            app.show_toast(app.recognition(), Some(("Aggiungi dettagli".into(), e.id)));
        }
    };
    let strain = move |s: StrainPreset| {
        let payload = Payload::Cannabis(Cannabis {
            preset_id: Some(s.id),
            strain: s.name.clone(),
            thc_pct: s.thc_pct,
            cbd_pct: s.cbd_pct,
            amount: s.amount,
            unit: s.unit,
            route: s.route,
        });
        if let Some(e) = app.record(payload) {
            app.show_toast(
                format!("{}: segnato alle {}.", s.name, time::hhmm(&e.ts)),
                Some(("Modifica".into(), e.id)),
            );
        }
    };
    let supplement = move |s: SupplementPreset| {
        let payload = Payload::Supplement(Supplement {
            preset_id: Some(s.id),
            name: s.name.clone(),
            amount: s.amount,
            unit: s.unit,
        });
        if let Some(e) = app.record(payload) {
            app.show_toast(
                format!("{}: segnato alle {}.", s.name, time::hhmm(&e.ts)),
                Some(("Modifica".into(), e.id)),
            );
        }
    };
    let save_reflection = move |e: FormEvent| {
        e.prevent_default();
        let text = reflection_text.read().trim().to_owned();
        if text.is_empty() {
            return;
        }
        if app
            .record(Payload::Reflection(Reflection { text }))
            .is_some()
        {
            reflection_text.set(String::new());
            app.show_toast(app.recognition(), None);
        }
    };

    rsx! {
        section { class: "fade",
            p { class: "muted", {long_date(&now)} }
            h1 { {greeting(now.hour())} }

            div { class: "card",
                h2 { class: "first", "Come stai?" }
                p { class: "muted", "Cinque domande veloci, tutte facoltative." }
                button { class: "primary wide", onclick: move |_| app.go(View::Checkin), "Fai il check-in" }
            }

            h2 { "Un gesto di cura" }
            p { class: "muted", "Un tocco per segnare qualcosa di buono che hai fatto per te." }
            div { class: "chips",
                for l in lists.care_labels.iter().cloned() {
                    button { key: "{l}", class: "chip", onclick: move |_| care(l.clone()), "{l}" }
                }
            }

            h2 { "La casa" }
            p { class: "muted", "Anche sistemare casa è prendersi cura di sé." }
            div { class: "chips",
                for r in lists.rooms.iter().cloned() {
                    button { key: "{r}", class: "chip", onclick: move |_| room(r.clone()), "{r}" }
                }
            }

            h2 { "Assunzioni" }
            if !thc.is_empty() {
                p { class: "label", "Prevalenza THC" }
                div { class: "chips",
                    for s in thc {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| strain(s.clone()) }, "{s.name}" }
                    }
                }
            }
            if !cbd.is_empty() {
                p { class: "label", "Prevalenza CBD" }
                div { class: "chips",
                    for s in cbd {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| strain(s.clone()) }, "{s.name}" }
                    }
                }
            }
            if !lists.supplements.is_empty() {
                p { class: "label", "Integratori" }
                div { class: "chips",
                    for s in lists.supplements.iter().cloned() {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| supplement(s.clone()) }, "{s.name}" }
                    }
                }
            }

            h2 { "Pasti" }
            div { class: "chips",
                for t in MealType::ALL {
                    button { key: "{meal_label(t)}", class: "chip", onclick: move |_| app.go(View::Meal(Some(t))), {meal_label(t)} }
                }
            }

            div { class: "row spaced",
                button { onclick: move |_| app.go(View::Thought), "Un pensiero che torna" }
            }

            if show_meal_recall {
                div { class: "card soft",
                    p { "Ti va di segnare cosa hai mangiato oggi?" }
                    div { class: "chips",
                        for t in meals_left {
                            button { key: "{meal_label(t)}", class: "chip", onclick: move |_| app.go(View::Meal(Some(t))), {meal_label(t)} }
                        }
                    }
                    button {
                        class: "quiet small",
                        onclick: move |_| {
                            let today = time::today();
                            app.update_settings(|s| s.meal_recall_dismissed_on = Some(today));
                        },
                        "Non ora"
                    }
                }
            }

            if evening {
                div { class: "card soft",
                    h2 { class: "first", "Cosa ti ha fatto bene oggi?" }
                    if let Some(r) = reflection {
                        if let Payload::Reflection(p) = &r.payload { p { "{p.text}" } }
                        button { class: "quiet small", onclick: move |_| app.go(View::Edit(r.id.clone())), "Cambia" }
                    } else {
                        form { onsubmit: save_reflection,
                            input {
                                r#type: "text",
                                maxlength: "200",
                                aria_label: "Cosa ti ha fatto bene oggi",
                                placeholder: "Una riga, se ti va",
                                value: "{reflection_text}",
                                oninput: move |e| reflection_text.set(e.value()),
                            }
                            p { class: "hint", "Resta solo su questo telefono." }
                            button { class: "small", r#type: "submit", disabled: reflection_text.read().trim().is_empty(), "Tienilo" }
                        }
                    }
                }
            }

            if !todays.is_empty() {
                h2 { "Oggi" }
                ul { class: "today",
                    for e in todays {
                        li { key: "{e.id}",
                            button {
                                class: "entry",
                                onclick: { let id = e.id.clone(); move |_| app.go(View::Edit(id.clone())) },
                                span { class: "time", {time::hhmm(&e.ts)} }
                                " {describe(&e)}"
                            }
                        }
                    }
                }
            }

            div { class: "actions",
                button { onclick: move |_| app.go(View::Enrich), "Se ti va: un momento per sistemare" }
                button { class: "quiet", onclick: move |_| app.lock(), "Chiudi Kuracare" }
            }
        }
    }
}
