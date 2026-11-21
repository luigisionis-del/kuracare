//! R1 smoke app: proves that the Rust core works on the Pixel
//! (SQLite file, Argon2id cost, time zone) before the full UI (R2).

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use dioxus::prelude::*;
use kuracare_core::kinds::Care;
use kuracare_core::{Event, Origin, Payload, SqliteStore, Vault, repo, time};

static VAULT: OnceLock<Arc<Vault<SqliteStore>>> = OnceLock::new();

/// App-private directory. On Android this is the sandboxed data dir of the package.
fn data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/org.kuracare.app/files")
    }
    #[cfg(not(target_os = "android"))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kuracare")
    }
}

fn vault() -> Arc<Vault<SqliteStore>> {
    VAULT
        .get_or_init(|| {
            let dir = data_dir();
            std::fs::create_dir_all(&dir).expect("data dir");
            Arc::new(Vault::new(
                SqliteStore::open(dir.join("kuracare.db")).expect("open db"),
            ))
        })
        .clone()
}

const CSS: &str = r#"
:root { color-scheme: light dark; --bg:#f6f1ea; --surface:#fffdf9; --text:#2a2622; --muted:#5f574e; --accent:#3f6b5a; --accent-text:#fff; --border:#ddd2c4; --danger:#9a3b2e; }
@media (prefers-color-scheme: dark) { :root { --bg:#161b18; --surface:#1f2622; --text:#ece6dd; --muted:#b3aa9f; --accent:#8cc4ab; --accent-text:#10231b; --border:#3a443e; --danger:#f0a092; } }
* { box-sizing: border-box; }
html, body { margin:0; background:var(--bg); color:var(--text); font-family:system-ui, sans-serif; font-size:17px; line-height:1.5; }
main { max-width:34rem; margin:0 auto; padding:1.5rem 1rem 3rem; }
h1 { font-size:1.6rem; margin:.5rem 0 1rem; }
.card { background:var(--surface); border:1px solid var(--border); border-radius:16px; padding:1rem; margin:0 0 1rem; }
.muted { color:var(--muted); }
.error { color:var(--danger); font-weight:600; }
input { width:100%; min-height:52px; padding:0 .9rem; border-radius:12px; border:1px solid var(--border); background:var(--surface); color:var(--text); font:inherit; }
button { min-height:52px; padding:0 1.25rem; border-radius:999px; border:1px solid var(--border); background:var(--surface); color:var(--text); font:inherit; font-weight:600; width:100%; margin-top:.6rem; }
button.primary { background:var(--accent); color:var(--accent-text); border-color:var(--accent); }
code { font-size:.85rem; word-break:break-all; }
"#;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq)]
enum Phase {
    New,
    Locked,
    Open,
}

#[component]
fn App() -> Element {
    let mut phase = use_signal(|| {
        if vault().exists().unwrap_or(false) {
            Phase::Locked
        } else {
            Phase::New
        }
    });
    let mut passphrase = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut kdf_ms = use_signal(|| None::<u128>);
    let mut events = use_signal(Vec::<Event>::new);

    let mut open = move |create: bool| {
        let pass = passphrase.read().clone();
        busy.set(true);
        error.set(None);
        spawn(async move {
            let started = Instant::now();
            let res = tokio::task::spawn_blocking(move || {
                let v = vault();
                if create {
                    v.create(&pass)
                } else {
                    v.unlock(&pass)
                }?;
                repo::load(&v)
            })
            .await
            .expect("worker thread");
            kdf_ms.set(Some(started.elapsed().as_millis()));
            busy.set(false);
            passphrase.set(String::new());
            match res {
                Ok(list) => {
                    events.set(list);
                    phase.set(Phase::Open);
                }
                Err(kuracare_core::Error::WrongPassphrase) => {
                    error.set(Some("Non sembra la passphrase giusta.".into()))
                }
                Err(e) => error.set(Some(format!("Qualcosa non è andato: {e}"))),
            }
        });
    };

    let add_walk = move |_| {
        let v = vault();
        let e = Event::new(
            Payload::Care(Care {
                label: "Passeggiata".into(),
                duration_min: None,
                fulfilment: None,
            }),
            Origin::Spontaneous,
        );
        match repo::save(&v, &e).and_then(|_| repo::load(&v)) {
            Ok(list) => events.set(list),
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    let now = time::now();
    rsx! {
        style { {CSS} }
        main {
            p { class: "muted", "Prova del nucleo Rust (R1)" }
            match phase() {
                Phase::New | Phase::Locked => rsx! {
                    h1 { if phase() == Phase::New { "Scegli una passphrase" } else { "Che bello rivederti" } }
                    input {
                        r#type: "password",
                        aria_label: "Passphrase",
                        value: "{passphrase}",
                        oninput: move |e| passphrase.set(e.value()),
                    }
                    if let Some(err) = error() { p { class: "error", "{err}" } }
                    button {
                        class: "primary",
                        disabled: busy() || passphrase.read().chars().count() < 10,
                        onclick: move |_| open(phase() == Phase::New),
                        if busy() { "Preparo la chiave…" } else if phase() == Phase::New { "Crea" } else { "Apri" }
                    }
                    p { class: "muted", "Almeno 10 caratteri." }
                },
                Phase::Open => rsx! {
                    h1 { "Aperta" }
                    div { class: "card",
                        p { "Chiave pronta in {kdf_ms().unwrap_or(0)} ms (Argon2id, 64 MiB)." }
                        p { "Fuso del telefono: " code { "{time::zone_name(&now)}" } }
                        p { "Ora: " code { "{time::format(&now)}" } }
                        p { "Registrazioni salvate: {events.read().len()}" }
                    }
                    button { class: "primary", onclick: add_walk, "Segna una passeggiata" }
                    for e in events.read().iter().take(5) {
                        p { class: "muted", code { "{e.ts}" } }
                    }
                    button {
                        onclick: move |_| {
                            vault().lock();
                            events.set(vec![]);
                            phase.set(Phase::Locked);
                        },
                        "Chiudi"
                    }
                },
            }
        }
    }
}
