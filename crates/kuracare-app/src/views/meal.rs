use dioxus::prelude::*;
use kuracare_core::kinds::{Meal as MealPayload, MealType, Payload};
use kuracare_core::repo;

use crate::labels::meal_label;
use crate::photo;
use crate::platform::vault;
use crate::state::use_app;
use crate::views::parts::Chip;

/// Reads the chosen file, shrinks it off the UI thread. Returns JPEG bytes.
pub async fn read_photo(e: &FormEvent) -> Option<Result<Vec<u8>, String>> {
    let file = e.files().into_iter().next()?;
    let bytes = match file.read_bytes().await {
        Ok(b) => b.to_vec(),
        Err(err) => return Some(Err(err.to_string())),
    };
    Some(
        tokio::task::spawn_blocking(move || photo::shrink(&bytes))
            .await
            .unwrap_or_else(|e| Err(e.to_string())),
    )
}

#[component]
pub fn Meal(meal_type: Option<MealType>) -> Element {
    let app = use_app();
    let mut kind = use_signal(|| meal_type.unwrap_or(MealType::Lunch));
    let mut description = use_signal(String::new);
    let mut jpeg = use_signal(|| None::<Vec<u8>>);
    let mut reading = use_signal(|| false);

    let pick = move |e: FormEvent| {
        reading.set(true);
        spawn(async move {
            match read_photo(&e).await {
                Some(Ok(b)) => jpeg.set(Some(b)),
                Some(Err(err)) => app.error(err),
                None => {}
            }
            reading.set(false);
        });
    };

    let save = move |_| {
        let photo_id = match jpeg.read().as_deref() {
            Some(b) => match repo::save_photo(&vault(), b) {
                Ok(id) => Some(id),
                Err(err) => return app.error(err),
            },
            None => None,
        };
        let d = description.read().trim().to_owned();
        let payload = Payload::Meal(MealPayload {
            meal_type: kind(),
            description: (!d.is_empty()).then_some(d),
            photo_id,
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
            label { r#for: "mp", "Oppure una foto " span { class: "muted", "(resta solo sul telefono)" } }
            input { id: "mp", class: "file", r#type: "file", accept: "image/*", capture: "environment", onchange: pick }
            if reading() { p { class: "hint", "Preparo la foto…" } }
            if let Some(b) = jpeg.read().as_deref() {
                img { class: "photo", src: photo::data_url(b), alt: "Foto del pasto" }
            }
            div { class: "actions",
                button { class: "primary", disabled: reading(), onclick: save, "Segna" }
                button { class: "quiet", onclick: move |_| app.back(), "Non ora" }
            }
        }
    }
}
