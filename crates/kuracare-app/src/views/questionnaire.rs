//! A versioned questionnaire (shown only when its definition is `ready`).

use dioxus::prelude::*;
use kuracare_core::Origin;
use kuracare_core::kinds::{Payload, Questionnaire as Answers};
use kuracare_core::questionnaires::{self, score};

use crate::state::use_app;
use crate::views::parts::{Chip, ScreenHead};

#[component]
pub fn Questionnaire(id: String) -> Element {
    let app = use_app();
    let def = use_hook(|| {
        questionnaires::builtin()
            .into_iter()
            .find(|d| d.id == id && d.ready)
    });
    let Some(def) = def else {
        return rsx! {
            p { class: "muted", "Questo questionario non è disponibile." }
            button { onclick: move |_| app.back(), "Torna indietro" }
        };
    };
    let n = def.items.len();
    let mut answers = use_signal(|| vec![None::<u8>; n]);

    let save = {
        let def = def.clone();
        move |_| {
            let a = answers.read().clone();
            let payload = Payload::Questionnaire(Answers {
                instrument: def.id.clone(),
                version: def.version.clone(),
                score: score(&def, &a),
                answers: a,
            });
            if app.record_as(payload, Origin::Prompted).is_some() {
                app.show_toast("Grazie: le risposte restano qui.", None);
                app.back();
            }
        }
    };

    rsx! {
        section { class: "fade",
            ScreenHead { title: def.name.clone(), icon: "checkin", subtitle: def.intro.clone() }
            for (i, item) in def.items.iter().enumerate() {
                div { key: "{i}", class: "card q-item",
                    p { class: "label", "{item}" }
                    div { class: "chips", role: "group", aria_label: "{item}",
                        for o in def.options.iter().cloned() {
                            Chip {
                                key: "{o.value}",
                                label: o.label.clone(),
                                pressed: answers.read()[i] == Some(o.value),
                                onclick: move |_| {
                                    let mut a = answers.write();
                                    a[i] = if a[i] == Some(o.value) { None } else { Some(o.value) };
                                },
                            }
                        }
                    }
                }
            }
            div { class: "actions dock",
                button { class: "primary", onclick: save, "Fatto" }
                button { class: "quiet", onclick: move |_| app.back(), "Non ora" }
            }
        }
    }
}
