use dioxus::prelude::*;
use kuracare_core::Origin;
use kuracare_core::kinds::{Checkin as CheckinPayload, CheckinItem, Payload};

use crate::labels::checkin_label;
use crate::state::use_app;
use crate::views::parts::Scale;

#[component]
pub fn Checkin(origin: Origin) -> Element {
    let app = use_app();
    let mut answers = use_signal(CheckinPayload::empty);

    let save = move |_| {
        if app
            .record_as(Payload::Checkin(answers.read().clone()), origin)
            .is_some()
        {
            app.show_toast(app.recognition(), None);
            app.back();
        }
    };

    rsx! {
        section { class: "fade",
            h1 { "Come stai, adesso?" }
            p { class: "muted", "Rispondi solo a ciò che ti va: ogni voce si può lasciare vuota." }
            for item in CheckinItem::ALL {
                Scale {
                    key: "{item:?}",
                    id: format!("ci-{item:?}"),
                    label: checkin_label(item).0,
                    low: checkin_label(item).1,
                    high: checkin_label(item).2,
                    value: item.get(&answers.read()),
                    onchange: move |v| item.set(&mut answers.write(), v),
                }
            }
            div { class: "actions",
                button { class: "primary", onclick: save, "Fatto" }
                button { class: "quiet", onclick: move |_| app.back(), "Non ora" }
            }
        }
    }
}
