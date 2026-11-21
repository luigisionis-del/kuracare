use dioxus::prelude::*;
use kuracare_core::kinds::{Payload, Thought as ThoughtPayload};

use crate::state::use_app;
use crate::views::parts::Chip;

#[component]
pub fn Thought() -> Element {
    let app = use_app();
    let mut label = use_signal(|| None::<String>);
    let mut note = use_signal(String::new);
    let labels = app.lists.read().thought_labels.clone();

    let save = move |_| {
        let n = note.read().trim().to_owned();
        let payload = Payload::Thought(ThoughtPayload {
            present: true,
            label: label.read().clone(),
            note: (!n.is_empty()).then_some(n),
        });
        if app.record(payload).is_some() {
            app.show_toast("Annotato. Lasciarlo qui può già alleggerire.", None);
            app.back();
        }
    };

    rsx! {
        section { class: "fade",
            h1 { "Un pensiero che torna" }
            p { class: "muted", "Se vuoi, dagli un nome. Non serve altro." }
            div { class: "chips", role: "group", aria_label: "Che tipo di pensiero",
                for l in labels {
                    Chip {
                        key: "{l}",
                        label: l.clone(),
                        pressed: label.read().as_deref() == Some(l.as_str()),
                        onclick: move |_| {
                            let same = label.read().as_deref() == Some(l.as_str());
                            label.set(if same { None } else { Some(l.clone()) });
                        },
                    }
                }
            }
            label { r#for: "tn", "Due parole, se ti va " span { class: "muted", "(restano solo qui)" } }
            input { id: "tn", r#type: "text", maxlength: "280", autocomplete: "off", value: "{note}", oninput: move |e| note.set(e.value()) }
            div { class: "actions",
                button { class: "primary", onclick: save, "Segna" }
                button { class: "quiet", onclick: move |_| app.back(), "Non ora" }
            }
        }
    }
}
