//! "I tuoi dati": encrypted backup, research export with a preview of what
//! leaves, the log of every release ("Invio dati"), total deletion.

use dioxus::prelude::*;
use kuracare_core::kinds::{Consent, DataRelease, Payload, research_fields};
use kuracare_core::{Origin, backup, export, time};

use crate::labels::{describe, kind_label};
use crate::notify::{self, Doc};
use crate::platform::{cache_dir, vault};
use crate::state::{Phase, use_app};
use crate::theme::Look;
use crate::views::parts::{Chip, Icon, ScreenHead};

const CONFIRM_WORD: &str = "CANCELLA";

#[derive(Clone, Copy, PartialEq)]
enum Period {
    All,
    Weeks4,
    Months3,
}

impl Period {
    const ALL: [Period; 3] = [Self::Weeks4, Self::Months3, Self::All];
    fn label(self) -> &'static str {
        match self {
            Self::Weeks4 => "Ultime 4 settimane",
            Self::Months3 => "Ultimi 3 mesi",
            Self::All => "Tutto",
        }
    }
    fn from(self) -> Option<jiff::civil::Date> {
        let days = match self {
            Self::Weeks4 => 27,
            Self::Months3 => 91,
            Self::All => return None,
        };
        time::now()
            .date()
            .checked_sub(jiff::Span::new().days(days))
            .ok()
    }
}

/// Writes `bytes` to the app cache, asks where to save it, waits for the answer.
async fn save_out(bytes: Vec<u8>, name: &str, mime: &str) -> Doc {
    let dir = cache_dir();
    let path = dir.join(name);
    if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, &bytes)) {
        return Doc::Failed(e.to_string());
    }
    notify::save_document(&path.to_string_lossy(), mime, name);
    let res = notify::wait_doc().await;
    let _ = std::fs::remove_file(&path);
    res
}

#[component]
pub fn Data() -> Element {
    let mut app = use_app();
    let mut busy = use_signal(|| false);
    let mut period = use_signal(|| Period::Weeks4);
    let mut csv = use_signal(|| true);
    let mut checked = use_signal(|| false);
    let mut confirm = use_signal(String::new);
    let mut wipe_open = use_signal(|| false);

    let tz = jiff::tz::TimeZone::system();
    let events = app.events.read().clone();
    let today = time::now().date();
    let from = period().from();
    let selected: Vec<_> = export::select(&events, from, Some(today), &tz)
        .into_iter()
        .filter(|e| !research_fields(e.kind()).is_empty())
        .collect();
    let preview = export::preview(&selected);
    let total = selected.len();
    let releases: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.payload, Payload::DataRelease(_)))
        .cloned()
        .collect();

    let log = move |r: DataRelease| {
        app.record_as(Payload::DataRelease(r), Origin::Spontaneous);
    };

    let make_backup = move |_| {
        busy.set(true);
        spawn(async move {
            let bytes = match backup::create(vault().store()) {
                Ok(b) => b,
                Err(e) => {
                    app.error(e);
                    busy.set(false);
                    return;
                }
            };
            let records = backup::info(&bytes).map(|i| i.records).unwrap_or(0);
            let name = format!("kuracare-backup-{}.kuracare", time::today());
            match save_out(bytes, &name, "application/octet-stream").await {
                Doc::Saved(dest) => {
                    log(DataRelease {
                        scope: "backup".into(),
                        format: "kuracare-backup".into(),
                        from: None,
                        to: Some(time::today()),
                        kinds: vec![],
                        records: records as u32,
                        destination: dest.clone(),
                        consent: Consent::Given,
                    });
                    app.show_toast(format!("Backup salvato: {dest}. Tienilo al sicuro."), None);
                }
                Doc::Cancelled => app.show_toast("Va bene, nessun file salvato.", None),
                Doc::Failed(e) | Doc::Opened(e) => app.error(e),
            }
            busy.set(false);
        });
    };

    let make_export = move |_| {
        busy.set(true);
        let tz = jiff::tz::TimeZone::system();
        let events = app.events.peek().clone();
        let today = time::now().date();
        let from = period().from();
        let as_csv = csv();
        spawn(async move {
            let sel: Vec<_> = export::select(&events, from, Some(today), &tz)
                .into_iter()
                .filter(|e| !research_fields(e.kind()).is_empty())
                .collect();
            let kinds: Vec<String> = export::preview(&sel)
                .into_iter()
                .map(|(k, _, _)| k)
                .collect();
            let (bytes, ext, mime) = if as_csv {
                (
                    export::research_csv(&sel).map(String::into_bytes),
                    "csv",
                    "text/csv",
                )
            } else {
                (export::research_json(&sel), "json", "application/json")
            };
            let bytes = match bytes {
                Ok(b) => b,
                Err(e) => {
                    app.error(e);
                    busy.set(false);
                    return;
                }
            };
            let name = format!("kuracare-ricerca-{}.{ext}", time::today());
            match save_out(bytes, &name, mime).await {
                Doc::Saved(dest) => {
                    log(DataRelease {
                        scope: "research".into(),
                        format: ext.into(),
                        from: from.map(|d| d.to_string()),
                        to: Some(today.to_string()),
                        kinds,
                        records: sel.len() as u32,
                        destination: dest.clone(),
                        consent: Consent::Given,
                    });
                    checked.set(false);
                    app.show_toast(format!("Esportazione salvata: {dest}."), None);
                }
                Doc::Cancelled => app.show_toast("Va bene, nessun file salvato.", None),
                Doc::Failed(e) | Doc::Opened(e) => app.error(e),
            }
            busy.set(false);
        });
    };

    let wipe = move |_| {
        match vault().destroy() {
            Ok(()) => {
                // Invitations off, appearance back to default, no files left behind.
                notify::set_plan(
                    &kuracare_core::reminders::plan(&Default::default(), &[], &time::now())
                        .to_json(),
                );
                let _ = std::fs::remove_file(crate::platform::data_dir().join("look.json"));
                crate::platform::clear_exchange_files();
                app.set_look(Look::default());
                // Drops everything held in memory (events, lists, settings).
                app.lock();
                app.phase.set(Phase::Onboarding);
            }
            Err(e) => app.error(e),
        }
    };

    rsx! {
        section { class: "fade",
            ScreenHead {
                title: "I tuoi dati",
                icon: "lock",
                subtitle: "Escono dal telefono solo così, quando lo decidi tu. Ogni uscita resta annotata qui sotto.",
            }

            div { class: "card group",
                h2 { class: "first sec", Icon { name: "lock" } "Backup cifrato" }
                p { class: "muted",
                    "Una copia completa, cifrata con la tua passphrase di adesso. Serve per non perdere nulla "
                    "se cambi telefono o se qualcosa va storto: si ripristina dal primo avvio di Kuracare."
                }
                p { class: "hint", "Senza la passphrase il backup non si apre, da nessuno. Tienilo dove vuoi, ma ricorda la frase." }
                button { class: "primary", disabled: busy(), onclick: make_backup, "Crea un backup" }
            }

            div { class: "card group",
                h2 { class: "first sec", Icon { name: "sparkle" } "Esportazione per la ricerca" }
                p { class: "muted",
                    "Solo dati standard, senza testi liberi: niente riflessioni, note, titoli delle cose da fare, descrizioni dei pasti, foto."
                }
                p { class: "label", "Periodo" }
                div { class: "chips", role: "group", aria_label: "Periodo",
                    for p in Period::ALL {
                        Chip { key: "{p.label()}", label: p.label(), pressed: period() == p, onclick: move |_| { period.set(p); checked.set(false); } }
                    }
                }
                p { class: "label", "Formato" }
                div { class: "chips", role: "group", aria_label: "Formato",
                    Chip { label: "CSV (fogli di calcolo)", pressed: csv(), onclick: move |_| csv.set(true) }
                    Chip { label: "JSON", pressed: !csv(), onclick: move |_| csv.set(false) }
                }
                p { class: "label",
                    if total == 1 { "Cosa esce (1 registrazione)" } else { "Cosa esce ({total} registrazioni)" }
                }
                if preview.is_empty() {
                    p { class: "hint", "In questo periodo non c'è niente da esportare." }
                } else {
                    ul { class: "plain preview",
                        for (kind, n, fields) in preview.iter() {
                            li { key: "{kind}",
                                strong { "{kind_label(kind)}: {n}" }
                                span { class: "muted", " · data e ora, fuso, origine, " {fields.join(", ")} }
                            }
                        }
                    }
                    label { class: "check",
                        input { r#type: "checkbox", checked: checked(), onchange: move |e| checked.set(e.checked()) }
                        span { "Ho controllato cosa esce." }
                    }
                    button {
                        class: "primary",
                        disabled: busy() || !checked(),
                        onclick: make_export,
                        "Scegli dove salvare il file"
                    }
                }
            }

            div { class: "card group",
                h2 { class: "first sec", Icon { name: "list" } "Cosa è uscito e quando" }
                if releases.is_empty() {
                    p { class: "muted", "Finora nessun dato è uscito da questo telefono." }
                }
                ul { class: "today",
                    for e in releases {
                        li { key: "{e.id}",
                            div { class: "entry static",
                                span { class: "time", {format!("{} {}", time::local_day(&e.ts), time::hhmm(&e.ts))} }
                                span {
                                    {describe(&e)}
                                    if let Payload::DataRelease(r) = &e.payload {
                                        span { class: "muted",
                                            {
                                                let what = if r.scope == "backup" { "elementi cifrati" } else { "registrazioni" };
                                                format!(" · {} {what} · «{}»", r.records, r.destination)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "card group danger-zone",
                h2 { class: "first", "Cancellare tutto" }
                p { class: "muted", "Cancella ogni dato di Kuracare da questo telefono: registrazioni, liste, impostazioni, inviti. I backup che hai salvato altrove restano dove sono." }
                if !wipe_open() {
                    button { class: "danger", onclick: move |_| wipe_open.set(true), "Voglio cancellare tutto" }
                } else {
                    p { strong { "Non si può annullare." } }
                    label { r#for: "wc", "Per confermare scrivi {CONFIRM_WORD}" }
                    input { id: "wc", r#type: "text", autocomplete: "off", autocapitalize: "characters", value: "{confirm}", oninput: move |e| confirm.set(e.value()) }
                    div { class: "actions",
                        button { class: "danger", disabled: *confirm.read() != CONFIRM_WORD, onclick: wipe, "Cancella tutto" }
                        button { class: "quiet", onclick: move |_| { wipe_open.set(false); confirm.set(String::new()); }, "Ci ho ripensato" }
                    }
                }
            }
        }
    }
}
