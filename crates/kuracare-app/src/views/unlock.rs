use dioxus::prelude::*;

use crate::platform::vault;
use crate::state::{Phase, use_app};

const CONFIRM_WORD: &str = "CANCELLA";

#[component]
pub fn Unlock() -> Element {
    let mut app = use_app();
    let mut pass = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut forgot = use_signal(|| false);
    let mut confirm = use_signal(String::new);

    let submit = move |e: FormEvent| {
        e.prevent_default();
        let p = pass.read().clone();
        if p.is_empty() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn(async move {
            let started = std::time::Instant::now();
            let res = tokio::task::spawn_blocking(move || vault().unlock(&p))
                .await
                .expect("worker thread");
            // Visible in logcat (RustStdoutStderr): only the duration, nothing personal.
            eprintln!(
                "kuracare: key derivation took {} ms",
                started.elapsed().as_millis()
            );
            busy.set(false);
            pass.set(String::new());
            match res.and_then(|_| app.opened()) {
                Ok(()) => {}
                Err(kuracare_core::Error::WrongPassphrase) => error.set(Some(
                    "Non sembra la passphrase giusta. Riprova con calma.".into(),
                )),
                Err(e) => error.set(Some(format!("Qualcosa non è andato: {e}"))),
            }
        });
    };

    let wipe = move |_| match vault().destroy() {
        Ok(()) => {
            forgot.set(false);
            confirm.set(String::new());
            app.phase.set(Phase::Onboarding);
        }
        Err(e) => error.set(Some(format!("Qualcosa non è andato: {e}"))),
    };

    rsx! {
        if !forgot() {
            section { class: "fade",
                h1 { "Che bello rivederti" }
                p { class: "muted", "Kuracare è chiusa per proteggere i tuoi dati." }
                form { onsubmit: submit,
                    label { r#for: "pp", "Passphrase" }
                    input {
                        id: "pp",
                        r#type: "password",
                        autocomplete: "current-password",
                        value: "{pass}",
                        oninput: move |e| pass.set(e.value()),
                    }
                    if let Some(err) = error() {
                        p { class: "error", role: "alert", "{err}" }
                    }
                    div { class: "actions",
                        button { class: "primary", r#type: "submit", disabled: busy() || pass.read().is_empty(),
                            if busy() { "Apro…" } else { "Apri" }
                        }
                        button { class: "quiet", r#type: "button", onclick: move |_| forgot.set(true), "Ho dimenticato la passphrase" }
                    }
                }
            }
        } else {
            section { class: "fade",
                h1 { "Passphrase dimenticata" }
                p { "Mi dispiace: senza passphrase i dati non si possono aprire, da nessuno. È ciò che li tiene al sicuro." }
                p { "Se hai un backup e ricordi la passphrase con cui l'hai fatto, potrai ripartire da lì." }
                p {
                    "Altrimenti puoi cancellare tutto e ricominciare da capo. "
                    strong { "Non si può annullare." }
                }
                label { r#for: "c", "Per cancellare tutto scrivi {CONFIRM_WORD}" }
                input {
                    id: "c",
                    r#type: "text",
                    autocomplete: "off",
                    value: "{confirm}",
                    oninput: move |e| confirm.set(e.value()),
                }
                div { class: "actions",
                    button { class: "danger", disabled: *confirm.read() != CONFIRM_WORD, onclick: wipe, "Cancella tutto e ricomincia" }
                    button {
                        class: "quiet",
                        onclick: move |_| {
                            forgot.set(false);
                            confirm.set(String::new());
                        },
                        "Torna indietro"
                    }
                }
            }
        }
    }
}
