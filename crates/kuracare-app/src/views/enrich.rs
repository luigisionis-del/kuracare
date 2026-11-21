use dioxus::prelude::*;
use kuracare_core::{enrich, time};

use crate::labels::{describe, short_date};
use crate::state::{View, use_app};
use crate::views::parts::{Icon, ScreenHead, kind_icon};

#[component]
pub fn Enrich() -> Element {
    let app = use_app();
    let events = app.events.read().clone();
    let candidates: Vec<_> = enrich::candidates(&events, jiff::Timestamp::now())
        .into_iter()
        .cloned()
        .collect();

    rsx! {
        section { class: "fade",
            ScreenHead {
                title: "Se ti va",
                icon: "sparkle",
                subtitle: "Un angolo per i momenti tranquilli. Qui puoi aggiungere qualche dettaglio a ciò che hai segnato di fretta. Nulla è obbligatorio.",
            }
            if candidates.is_empty() {
                div { class: "card", p { "Qui non c'è niente da sistemare. Goditi il momento." } }
            }
            for e in candidates {
                div { key: "{e.id}", class: "card enrich-card",
                    p { class: "enrich-what",
                        Icon { name: kind_icon(&e.payload) }
                        strong { {describe(&e)} }
                    }
                    p { class: "muted",
                        {time::local(&e.ts).map(|z| short_date(&z)).unwrap_or_default()}
                        ", {time::hhmm(&e.ts)}"
                    }
                    div { class: "row",
                        button { class: "small", onclick: { let id = e.id.clone(); move |_| app.go(View::Edit(id.clone())) }, "Aggiungi dettagli" }
                        button {
                            class: "quiet small",
                            onclick: {
                                let e = e.clone();
                                move |_| {
                                    let mut s = e.clone();
                                    s.settled = true;
                                    app.change(&s);
                                }
                            },
                            "Va bene così"
                        }
                    }
                }
            }
            h2 { class: "sec", Icon { name: "list" } "Le tue liste" }
            p { class: "muted", "Varietà, integratori, gesti di cura, stanze, pensieri: puoi riordinarli con calma, per esempio copiando THC e CBD dalle etichette." }
            button { onclick: move |_| app.go(View::Lists), "Apri le tue liste" }
            div { class: "actions", button { class: "quiet", onclick: move |_| app.back(), "Torna a oggi" } }
        }
    }
}
