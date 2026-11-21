use dioxus::prelude::*;
use kuracare_core::reminders::{self, Invite, InviteKind, Plan};
use kuracare_core::settings::TimeWindow;
use kuracare_core::time;

use crate::labels::{OPEN_PAUSE_YEAR, pause_label};
use crate::notify;
use crate::state::{View, use_app};
use crate::views::parts::Chip;

const PAUSES: [(&str, i64); 3] = [("Un giorno", 1), ("Tre giorni", 3), ("Una settimana", 7)];
const LOCK_MINUTES: [u32; 5] = [1, 2, 5, 10, 15];

/// A test invitation, right away (tapping it just opens Kuracare).
fn test_plan() -> Plan {
    let now = jiff::Timestamp::now().as_millisecond();
    Plan {
        version: reminders::PLAN_VERSION,
        channel: reminders::CHANNEL,
        not_now: reminders::NOT_NOW,
        items: vec![Invite {
            key: format!("test:{now}"),
            kind: InviteKind::Checkin,
            at: now,
            until: now + 10 * 60 * 1000,
            title: reminders::CHECKIN_TITLE,
            body: reminders::CHECKIN_BODY,
        }],
    }
}

#[component]
pub fn Settings() -> Element {
    let app = use_app();
    let settings = app.settings.read().clone();
    let mut windows = use_signal(|| settings.checkin_windows.clone());
    let mut allowed = use_signal(notify::allowed);
    let windows_ok = windows.read().iter().all(TimeWindow::is_valid);
    let windows_changed = *windows.read() != settings.checkin_windows;
    let paused = settings.is_paused(jiff::Timestamp::now());

    // Android's permission dialog answers asynchronously: look again for a while.
    let recheck = move || {
        spawn(async move {
            for _ in 0..20 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                allowed.set(notify::allowed());
            }
        });
    };

    let toggle = move |_| {
        let on = !app.settings.peek().invitations;
        app.update_settings(|s| s.invitations = on);
        if on && !notify::allowed() {
            notify::ask();
            recheck();
        }
    };

    let pause_for = move |days: i64| {
        let until = time::now()
            .checked_add(jiff::SignedDuration::from_hours(24 * days))
            .map(|z| time::format(&z))
            .ok();
        app.update_settings(|s| s.pause_until = until);
    };

    rsx! {
        section { class: "fade",
            h1 { "Impostazioni" }

            h2 { "Inviti" }
            p { class: "muted",
                "Una notifica che chiede “Ti va di…?”, in un momento a caso di ogni finestra del check-in, "
                "e all'ora delle cose da fare che hanno un orario. Mai ripetuta: se non è il momento, "
                "tocca “Non ora” o lasciala lì, sparisce da sola."
            }
            if !notify::supported() {
                p { class: "hint", "Su questo dispositivo gli inviti non sono disponibili." }
            }
            label { class: "check",
                input { r#type: "checkbox", checked: settings.invitations, onchange: toggle }
                span { "Mandami gli inviti" }
            }
            if settings.invitations && notify::supported() {
                if !allowed() {
                    div { class: "card soft",
                        p { "Android non permette ancora le notifiche a Kuracare." }
                        button {
                            class: "small",
                            onclick: move |_| {
                                notify::open_settings();
                                recheck();
                            },
                            "Apri le impostazioni delle notifiche"
                        }
                    }
                } else {
                    button {
                        class: "small",
                        onclick: move |_| {
                            notify::show_now(&test_plan().to_json());
                            app.show_toast("Ecco com'è un invito: guarda tra le notifiche.", None);
                        },
                        "Mandami un invito di prova"
                    }
                }
                p { class: "hint",
                    "Il testo è sempre lo stesso e generico; sulla schermata di blocco si vede solo “Kuracare”. "
                    "Gli orari degli inviti restano sul telefono, fuori dalla cifratura, perché arrivino anche ad app chiusa."
                }
            }

            h2 { "Momenti del check-in" }
            for (i, w) in windows.read().iter().cloned().enumerate() {
                div { key: "{i}", class: "card row",
                    label { class: "sr", r#for: "ws{i}", "Dalle" }
                    input { id: "ws{i}", r#type: "time", value: "{w.start}", oninput: move |e| windows.write()[i].start = e.value() }
                    label { class: "sr", r#for: "we{i}", "alle" }
                    input { id: "we{i}", r#type: "time", value: "{w.end}", oninput: move |e| windows.write()[i].end = e.value() }
                    button { class: "quiet", aria_label: "Togli questo momento", onclick: move |_| { windows.write().remove(i); }, "Togli" }
                }
            }
            if !windows_ok {
                p { class: "hint", "Controlla gli orari: l'inizio deve venire prima della fine." }
            }
            div { class: "row",
                button {
                    class: "small",
                    onclick: move |_| windows.write().push(TimeWindow { start: "09:00".into(), end: "10:00".into() }),
                    "Aggiungi un momento"
                }
                if windows_changed {
                    button {
                        class: "small primary",
                        disabled: !windows_ok,
                        onclick: move |_| {
                            let w = windows.read().clone();
                            app.update_settings(|s| s.checkin_windows = w);
                            app.show_toast("Momenti aggiornati.", None);
                        },
                        "Salva i momenti"
                    }
                }
            }

            h2 { "Pausa" }
            if paused {
                p { "Sei in pausa {pause_label(settings.pause_until.as_deref())}. Kuracare non ti chiede niente e non manda inviti." }
                button { class: "small", onclick: move |_| app.update_settings(|s| s.pause_until = None), "Riprendi adesso" }
            } else {
                p { class: "muted", "Quando hai bisogno di silenzio: niente inviti e niente domande. Puoi sempre registrare, se ti va." }
                div { class: "chips",
                    for (label, days) in PAUSES {
                        button { key: "{label}", class: "chip", onclick: move |_| pause_for(days), "{label}" }
                    }
                    button {
                        class: "chip",
                        onclick: move |_| {
                            let until = format!("{OPEN_PAUSE_YEAR}-01-01T00:00:00.000+00:00");
                            app.update_settings(|s| s.pause_until = Some(until));
                        },
                        "Finché non la tolgo"
                    }
                }
            }

            h2 { "Blocco" }
            p { class: "muted", "Kuracare si chiude da sola dopo qualche minuto senza tocchi." }
            div { class: "chips", role: "group", aria_label: "Minuti prima del blocco",
                for m in LOCK_MINUTES {
                    Chip {
                        key: "{m}",
                        label: format!("{m} min"),
                        pressed: settings.lock_after_minutes == m,
                        onclick: move |_| app.update_settings(|s| s.lock_after_minutes = m),
                    }
                }
            }

            h2 { "Le tue liste" }
            p { class: "muted", "Gesti di cura, stanze, varietà, integratori, pensieri." }
            button { class: "small", onclick: move |_| app.go(View::Lists), "Apri le tue liste" }

            div { class: "actions",
                button { class: "quiet", onclick: move |_| app.back(), "Torna alla Home" }
            }
        }
    }
}
