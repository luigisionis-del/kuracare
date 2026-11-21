use dioxus::prelude::*;
use kuracare_core::settings::{self, Settings, TimeWindow};

use crate::platform::vault;
use crate::state::use_app;

const MIN_LENGTH: usize = 10;
const STEPS: usize = 5;

#[component]
pub fn Onboarding() -> Element {
    let app = use_app();
    let mut step = use_signal(|| 0usize);
    let mut pass = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut understood = use_signal(|| false);
    let mut windows = use_signal(Vec::<TimeWindow>::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let len = pass.read().chars().count();
    let pass_ok = len >= MIN_LENGTH && *pass.read() == *confirm.read() && understood();
    let windows_ok = windows.read().iter().all(TimeWindow::is_valid);

    let finish = move |_| {
        busy.set(true);
        error.set(None);
        let p = pass.read().clone();
        let s = Settings {
            checkin_windows: windows.read().clone(),
            ..Settings::default()
        };
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let v = vault();
                v.create(&p)?;
                settings::save(&v, &s)
            })
            .await
            .expect("worker thread");
            match res.and_then(|_| app.opened()) {
                Ok(()) => {
                    pass.set(String::new());
                    confirm.set(String::new());
                }
                Err(e) => {
                    error.set(Some(format!("Qualcosa non è andato: {e}")));
                    busy.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "dots", aria_hidden: "true",
            for i in 0..STEPS {
                span { key: "{i}", class: if i == step() { "on" } else { "" } }
            }
        }
        match step() {
            0 => rsx! {
                section { class: "fade",
                    h1 { "Ti do il benvenuto in Kuracare" }
                    p { "Un posto tranquillo per prenderti cura di te: le cose da fare, come stai, i gesti che ti fanno bene." }
                    p {
                        "Kuracare è un diario personale. Registra e riassume, ma "
                        strong { "non dà consigli sanitari" }
                        ": per quelli c'è chi ti segue."
                    }
                    div { class: "actions",
                        button { class: "primary", onclick: move |_| step.set(1), "Avanti" }
                    }
                }
            },
            1 => rsx! {
                section { class: "fade",
                    h1 { "Tutto resta qui" }
                    p { span { class: "badge-local", "Solo su questo telefono" } }
                    p { "Quello che scrivi resta su questo telefono, cifrato. Niente account, niente cloud, nessuno che guarda: Kuracare non si collega a internet." }
                    p { "I dati escono solo se lo decidi tu, con un'esportazione. Ogni volta Kuracare se lo annota, così sai sempre cosa è uscito e quando." }
                    div { class: "actions",
                        button { class: "primary", onclick: move |_| step.set(2), "Avanti" }
                        button { class: "quiet", onclick: move |_| step.set(0), "Indietro" }
                    }
                }
            },
            2 => rsx! {
                section { class: "fade",
                    h1 { "Una chiave solo tua" }
                    p {
                        "Ora sceglierai una "
                        strong { "passphrase" }
                        ": una frase che conosci solo tu. Serve a cifrare i dati e ad aprire l'app."
                    }
                    p {
                        strong { "Se la dimentichi, i dati non si possono recuperare" }
                        ", da nessuno: nemmeno da chi ha scritto l'app. È il prezzo della riservatezza."
                    }
                    p { class: "muted", "Più avanti potrai fare un backup cifrato, da tenere in un posto sicuro." }
                    div { class: "actions",
                        button { class: "primary", onclick: move |_| step.set(3), "Scelgo la passphrase" }
                        button { class: "quiet", onclick: move |_| step.set(1), "Indietro" }
                    }
                }
            },
            3 => rsx! {
                section { class: "fade",
                    h1 { "La tua passphrase" }
                    p { class: "muted", "Qualche parola che per te ha senso funziona meglio di una password complicata, per esempio quattro parole a caso." }
                    label { r#for: "pp", "Passphrase" }
                    input {
                        id: "pp",
                        r#type: "password",
                        autocomplete: "new-password",
                        value: "{pass}",
                        oninput: move |e| pass.set(e.value()),
                    }
                    p { class: "hint",
                        "Almeno {MIN_LENGTH} caratteri"
                        if len > 0 && len < MIN_LENGTH { " (ne mancano {MIN_LENGTH - len})" }
                        "."
                    }
                    label { r#for: "pp2", "Scrivila di nuovo" }
                    input {
                        id: "pp2",
                        r#type: "password",
                        autocomplete: "new-password",
                        value: "{confirm}",
                        oninput: move |e| confirm.set(e.value()),
                    }
                    if !confirm.read().is_empty() && *confirm.read() != *pass.read() {
                        p { class: "hint", "Le due frasi non coincidono ancora." }
                    }
                    label { class: "check",
                        input {
                            r#type: "checkbox",
                            checked: understood(),
                            onchange: move |e| understood.set(e.checked()),
                        }
                        span { "Ho capito: se dimentico la passphrase, i dati non si possono recuperare." }
                    }
                    div { class: "actions",
                        button { class: "primary", disabled: !pass_ok, onclick: move |_| step.set(4), "Avanti" }
                        button { class: "quiet", onclick: move |_| step.set(2), "Indietro" }
                    }
                }
            },
            _ => rsx! {
                section { class: "fade",
                    h1 { "Quando ti va di sentirti chiedere come stai?" }
                    p { "Puoi scegliere uno o più momenti della giornata. Saranno inviti, mai obblighi, e potrai cambiarli quando vuoi." }
                    for (i, w) in windows.read().iter().cloned().enumerate() {
                        div { key: "{i}", class: "card row",
                            label { class: "sr", r#for: "ws{i}", "Dalle" }
                            input {
                                id: "ws{i}",
                                r#type: "time",
                                value: "{w.start}",
                                oninput: move |e| windows.write()[i].start = e.value(),
                            }
                            label { class: "sr", r#for: "we{i}", "alle" }
                            input {
                                id: "we{i}",
                                r#type: "time",
                                value: "{w.end}",
                                oninput: move |e| windows.write()[i].end = e.value(),
                            }
                            button {
                                class: "quiet",
                                aria_label: "Togli questo momento",
                                onclick: move |_| { windows.write().remove(i); },
                                "Togli"
                            }
                        }
                    }
                    if !windows_ok {
                        p { class: "hint", "Controlla gli orari: l'inizio deve venire prima della fine." }
                    }
                    button {
                        onclick: move |_| windows.write().push(TimeWindow { start: "09:00".into(), end: "10:00".into() }),
                        "Aggiungi un momento"
                    }
                    if let Some(err) = error() {
                        p { class: "error", role: "alert", "{err}" }
                    }
                    div { class: "actions",
                        button {
                            class: "primary",
                            disabled: busy() || !windows_ok,
                            onclick: finish,
                            if busy() {
                                "Sto preparando la chiave…"
                            } else if windows.read().is_empty() {
                                "Per ora nessuno, iniziamo"
                            } else {
                                "Iniziamo"
                            }
                        }
                        button { class: "quiet", disabled: busy(), onclick: move |_| step.set(3), "Indietro" }
                    }
                }
            },
        }
    }
}
