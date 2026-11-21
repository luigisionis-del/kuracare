//! Restore from an encrypted backup, at first start (empty vault only).

use dioxus::prelude::*;
use kuracare_core::{Error, backup, time};

use crate::labels::day_label;
use crate::notify::{self, Doc};
use crate::platform::vault;
use crate::state::use_app;
use crate::views::parts::{Emblem, Icon};

#[component]
pub fn Restore(onback: EventHandler<()>) -> Element {
    let app = use_app();
    let mut file = use_signal(|| None::<(String, backup::Info)>);
    let mut pass = use_signal(String::new);
    let mut visible = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let pick = move |_| {
        error.set(None);
        busy.set(true);
        notify::open_document();
        spawn(async move {
            match notify::wait_doc().await {
                Doc::Opened(path) => match std::fs::read(&path)
                    .map_err(Error::from)
                    .and_then(|b| backup::info(&b))
                {
                    Ok(info) => file.set(Some((path, info))),
                    Err(e) => {
                        let _ = std::fs::remove_file(&path);
                        error.set(Some(e.to_string()));
                    }
                },
                Doc::Cancelled => {}
                Doc::Failed(e) | Doc::Saved(e) => {
                    error.set(Some(format!("Il file non si è aperto ({e}).")))
                }
            }
            busy.set(false);
        });
    };

    let restore = move |e: FormEvent| {
        e.prevent_default();
        let Some((path, _)) = file.read().clone() else {
            return;
        };
        let p = pass.read().clone();
        busy.set(true);
        error.set(None);
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let bytes = std::fs::read(&path)?;
                let v = vault();
                backup::restore(v.store(), &bytes)?;
                match v.unlock(&p) {
                    Ok(()) => {
                        let _ = std::fs::remove_file(&path);
                        Ok(())
                    }
                    Err(e) => {
                        // Never leave a half-open vault behind: back to empty.
                        let _ = v.destroy();
                        Err(e)
                    }
                }
            })
            .await
            .expect("worker thread");
            busy.set(false);
            pass.set(String::new());
            match res.and_then(|_| app.opened()) {
                Ok(()) => app.show_toast("Backup ripristinato: ritrovi tutto com'era.", None),
                Err(Error::WrongPassphrase) => error.set(Some(
                    "Non sembra la passphrase di questo backup. Serve quella che usavi quando l'hai creato.".into(),
                )),
                Err(e) => error.set(Some(format!("Qualcosa non è andato: {e}"))),
            }
        });
    };

    rsx! {
        section { class: "fade unlock",
            Emblem {}
            h1 { "Ripristina da un backup" }
            p { class: "muted", "Scegli il file del backup di Kuracare (finisce con .kuracare), poi scrivi la passphrase che usavi quando l'hai creato." }
            if let Some((_, info)) = file() {
                div { class: "card soft",
                    p { class: "enrich-what", Icon { name: "lock" } strong { "Backup trovato" } }
                    p { class: "muted",
                        "Creato {time::local(&info.created_at).map(|z| day_label(&z.strftime(\"%Y-%m-%d\").to_string())).unwrap_or_default()} · {info.records} elementi cifrati"
                    }
                }
                form { onsubmit: restore,
                    label { r#for: "rp", "Passphrase del backup" }
                    div { class: "pass-row",
                        input {
                            id: "rp",
                            r#type: if visible() { "text" } else { "password" },
                            autocomplete: "current-password",
                            autocapitalize: "off",
                            spellcheck: "false",
                            value: "{pass}",
                            oninput: move |e| pass.set(e.value()),
                        }
                        button {
                            class: "eye", r#type: "button",
                            aria_label: if visible() { "Nascondi la passphrase" } else { "Mostra la passphrase" },
                            aria_pressed: visible().to_string(),
                            onclick: move |_| visible.toggle(),
                            Icon { name: "eye" }
                        }
                    }
                    if let Some(err) = error() { p { class: "error", role: "alert", "{err}" } }
                    div { class: "actions",
                        button { class: "primary", r#type: "submit", disabled: busy() || pass.read().is_empty(),
                            if busy() { "Ripristino…" } else { "Ripristina" }
                        }
                        button { class: "quiet", r#type: "button", disabled: busy(), onclick: pick, "Scegli un altro file" }
                    }
                }
            } else {
                if let Some(err) = error() { p { class: "error", role: "alert", "{err}" } }
                div { class: "actions",
                    button { class: "primary", disabled: busy(), onclick: pick, "Scegli il file" }
                }
            }
            div { class: "actions",
                button { class: "quiet", disabled: busy(), onclick: move |_| onback.call(()), "Torna indietro" }
            }
        }
    }
}
