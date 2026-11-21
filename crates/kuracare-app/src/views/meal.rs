use dioxus::prelude::*;
use kuracare_core::kinds::{Meal as MealPayload, MealType, Payload};

use crate::labels::meal_label;
use crate::state::use_app;
use crate::views::parts::Chip;

#[component]
pub fn Meal(meal_type: Option<MealType>) -> Element {
    let app = use_app();
    let mut kind = use_signal(|| meal_type.unwrap_or(MealType::Lunch));
    let mut description = use_signal(String::new);

    let save = move |_| {
        let d = description.read().trim().to_owned();
        let payload = Payload::Meal(MealPayload {
            meal_type: kind(),
            description: (!d.is_empty()).then_some(d),
            // Photos are paused (21/11/2026): to be done later.
            photo_id: None,
        });
        if app.record(payload).is_some() {
            app.show_toast("Segnato. Buon appetito, anche a posteriori.", None);
            app.back();
        }
    };

    rsx! {
        section { class: "fade",
            h1 { "Un pasto" }
            div { class: "chips", role: "group", aria_label: "Quale pasto",
                for t in MealType::ALL {
                    Chip { key: "{t:?}", label: meal_label(t), pressed: kind() == t, onclick: move |_| kind.set(t) }
                }
            }
            label { r#for: "md", "Cosa c'era, in breve " span { class: "muted", "(facoltativo)" } }
            input { id: "md", r#type: "text", maxlength: "140", autocomplete: "off", value: "{description}", oninput: move |e| description.set(e.value()) }
            div { class: "actions",
                button { class: "primary", onclick: save, "Segna" }
                button { class: "quiet", onclick: move |_| app.back(), "Non ora" }
            }
        }
    }
}
