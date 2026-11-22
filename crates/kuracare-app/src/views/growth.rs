//! Crescita: the guided path through Covey's seven habits (texts in
//! `content/crescita/percorso.it.json`, logic in `kuracare_core::growth`).
//! Guided, never locked; no scores: only what was read, what was practised
//! and, if wanted, a self-assessment in words.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use kuracare_core::growth::{
    self, Dimension, GrowthPractice, GrowthRead, GrowthStage, Practice, PracticeKind, Quadrant,
    Stage, Step, StepStatus,
};
use kuracare_core::kinds::Payload;
use kuracare_core::summary::Counted;
use kuracare_core::{Event, time};

use crate::labels::{PATH, stage_label};
use crate::state::{View, use_app};
use crate::views::parts::{Chip, Icon, ScreenHead};
use crate::views::summary::Bars;

/// The groups of the book: the basics, the private victory, the public one, renewal.
const GROUPS: [(&str, u8, u8); 4] = [
    ("Prima delle abitudini", 0, 0),
    ("La vittoria privata", 1, 3),
    ("La vittoria pubblica", 4, 6),
    ("Il rinnovamento", 7, 7),
];

fn step_subtitle(s: &Step) -> String {
    if s.number == 0 {
        s.tagline.clone()
    } else {
        format!("Abitudine {} · {}", s.number, s.tagline)
    }
}

fn status_label(s: StepStatus) -> &'static str {
    match s {
        StepStatus::NotStarted => "da scoprire",
        StepStatus::Started => "iniziata",
        StepStatus::Explored => "esplorata",
    }
}

/// "2026-11-22" → "22/11/2026".
fn day_label(iso: &str) -> String {
    let d = time::local_day(iso);
    match (d.get(0..4), d.get(5..7), d.get(8..10)) {
        (Some(y), Some(m), Some(g)) => format!("{g}/{m}/{y}"),
        _ => d,
    }
}

fn choice_label(p: &Practice, id: &str) -> String {
    p.choices
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.label.clone())
        .unwrap_or_else(|| id.to_owned())
}

fn practices_of<'a>(events: &'a [Event], step: &str, practice: &str) -> Vec<&'a Event> {
    events
        .iter()
        .filter(|e| matches!(&e.payload, Payload::GrowthPractice(p) if p.step == step && p.practice == practice))
        .collect()
}

// ---- The path ------------------------------------------------------------

#[component]
pub fn Growth() -> Element {
    let app = use_app();
    let path = &*PATH;
    let events = app.events.read();
    let prog = growth::progress(path, &events);
    let next = growth::suggested(path, &events);
    let next_status = path
        .steps
        .iter()
        .zip(&prog)
        .find(|(s, _)| s.id == next.id)
        .map(|(_, p)| p.status)
        .unwrap_or(StepStatus::NotStarted);
    let nothing_yet = prog.iter().all(|p| p.status == StepStatus::NotStarted);
    let today = time::today();
    let review =
        if app.settings.read().growth_review_dismissed_on.as_deref() == Some(today.as_str()) {
            None
        } else {
            growth::due_reviews(path, &events, jiff::Timestamp::now())
                .into_iter()
                .next()
        };
    let mission = growth::mission(&events).map(|(e, p)| {
        let first = p
            .texts
            .get("chi")
            .or_else(|| p.texts.values().next())
            .cloned()
            .unwrap_or_default();
        (day_label(&e.ts), first)
    });
    drop(events);

    rsx! {
        section { class: "fade",
            ScreenHead {
                title: "Crescita",
                icon: "growth",
                subtitle: "Le sette abitudini, dalle idee di Stephen R. Covey",
            }

            if nothing_yet {
                div { class: "card soft",
                    p { class: "first",
                        "Un percorso in otto passi sulle idee di "
                        em { "Le 7 regole per avere successo" }
                        " (1989). Ogni passo ha delle schede da leggere e degli esercizi da provare nella tua vita di tutti i giorni."
                    }
                    p { class: "muted",
                        "I testi sono riassunti scritti con parole nostre, non sostituiscono il libro. Si va con i tuoi tempi: l'ordine è un consiglio, nessun passo è chiuso."
                    }
                }
            }

            if let Some(d) = review {
                if let Some(card) = path.step(&d.step).and_then(|s| s.card(&d.card)) {
                    div { class: "card",
                        p { class: "label first", "Ti va di ripassare?" }
                        p { "«{card.title}»" }
                        div { class: "row",
                            button {
                                class: "primary small",
                                onclick: {
                                    let (s, c) = (d.step.clone(), d.card.clone());
                                    move |_| app.go(View::GrowthCard(s.clone(), c.clone(), true))
                                },
                                "Ripassa"
                            }
                            button {
                                class: "quiet small",
                                onclick: move |_| {
                                    let today = time::today();
                                    app.update_settings(|s| s.growth_review_dismissed_on = Some(today));
                                },
                                "Non ora"
                            }
                        }
                    }
                }
            }

            div { class: "card week-card",
                div {
                    if nothing_yet {
                        p { class: "muted", "Si comincia da" }
                    } else if next_status == StepStatus::Explored {
                        p { class: "muted", "Hai esplorato tutto il percorso. Ora conta la pratica: torna sugli esercizi quando ti va." }
                    } else {
                        p { class: "muted", "Ti suggerisco di continuare con" }
                    }
                    if next_status != StepStatus::Explored {
                        p { strong { "{next.name}" } }
                    }
                }
                if next_status != StepStatus::Explored {
                    button {
                        class: "primary small",
                        onclick: { let id = next.id.clone(); move |_| app.go(View::GrowthStep(id.clone())) },
                        if nothing_yet { "Comincia" } else { "Continua" }
                    }
                }
            }

            if let Some((day, first)) = mission {
                div { class: "card soft",
                    p { class: "label", "La tua missione" span { class: "muted", " · {day}" } }
                    if !first.is_empty() { p { class: "mission", "{first}" } }
                    button {
                        class: "small",
                        onclick: move |_| app.go(View::GrowthPractice("fine-in-mente".into(), "missione".into())),
                        "Rileggi o cambia"
                    }
                }
            }

            for (title, from, to) in GROUPS {
                h2 { key: "{title}", class: "sec", "{title}" }
                ul { class: "plain",
                    for (s, p) in path.steps.iter().zip(&prog).filter(|(s, _)| (from..=to).contains(&s.number)) {
                        li { key: "{s.id}",
                            button {
                                class: "entry step-row",
                                onclick: { let id = s.id.clone(); move |_| app.go(View::GrowthStep(id.clone())) },
                                span { class: "step-num", "aria-hidden": "true",
                                    if s.number == 0 { "·" } else { "{s.number}" }
                                }
                                span { class: "step-text",
                                    strong { "{s.name}" }
                                    span { class: "muted", "{s.tagline}" }
                                }
                                span { class: "step-status",
                                    if p.status == StepStatus::Explored { Icon { name: "check" } }
                                    {p.stage.map(stage_label).map(str::to_lowercase).unwrap_or_else(|| status_label(p.status).into())}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---- A step ----------------------------------------------------------------

#[component]
pub fn StepView(step: String) -> Element {
    let app = use_app();
    let Some(st) = PATH.step(&step) else {
        return rsx! { p { "Questo passo non c'è più." } };
    };
    let events = app.events.read();
    let prog = growth::progress(&PATH, &events)
        .into_iter()
        .find(|p| p.step == st.id)
        .expect("progress of every step");
    let last_done: BTreeMap<&str, String> = st
        .practices
        .iter()
        .filter_map(|p| {
            practices_of(&events, &st.id, &p.id)
                .first()
                .map(|e| (p.id.as_str(), day_label(&e.ts)))
        })
        .collect();
    drop(events);
    let next = PATH.steps.iter().find(|s| s.number == st.number + 1);
    let stage = prog.stage;
    let first_unread = st.cards.iter().find(|c| !prog.cards_read.contains(&c.id));

    rsx! {
        section { class: "fade",
            ScreenHead { title: st.name.clone(), icon: "growth", subtitle: step_subtitle(st), up: "Crescita" }
            p { class: "idea", "{st.idea}" }

            h2 { class: "sec", Icon { name: "eye" } "Capire" }
            ul { class: "plain",
                for c in st.cards.iter() {
                    li { key: "{c.id}",
                        button {
                            class: "entry",
                            onclick: { let (s, id) = (st.id.clone(), c.id.clone()); move |_| app.go(View::GrowthCard(s.clone(), id.clone(), false)) },
                            if prog.cards_read.contains(&c.id) {
                                Icon { name: "check" }
                            } else {
                                span { class: "dot-new", "aria-hidden": "true" }
                            }
                            span { "{c.title}" }
                            if prog.cards_read.contains(&c.id) { span { class: "sr-only", " (letta)" } }
                        }
                    }
                }
            }
            if let Some(c) = first_unread {
                button {
                    class: "small",
                    onclick: { let (s, id) = (st.id.clone(), c.id.clone()); move |_| app.go(View::GrowthCard(s.clone(), id.clone(), false)) },
                    if prog.cards_read.is_empty() { "Comincia a leggere" } else { "Continua a leggere" }
                }
            }

            h2 { class: "sec", Icon { name: "care" } "Mettere in pratica" }
            ul { class: "plain",
                for p in st.practices.iter() {
                    li { key: "{p.id}",
                        button {
                            class: "entry step-row",
                            onclick: { let (s, id) = (st.id.clone(), p.id.clone()); move |_| app.go(View::GrowthPractice(s.clone(), id.clone())) },
                            span { class: "step-text",
                                strong { "{p.title}" }
                                if let Some(d) = last_done.get(p.id.as_str()) {
                                    span { class: "muted", "l'ultima volta il {d}" }
                                }
                            }
                        }
                    }
                }
            }

            h2 { class: "sec", Icon { name: "thought" } "Come la vivi" }
            p { class: "muted", "Se ti va. Puoi cambiarlo quando vuoi: serve solo a te." }
            div { class: "chips", role: "group", aria_label: "Come la vivi",
                for s in Stage::ALL {
                    Chip {
                        key: "{s:?}",
                        label: stage_label(s),
                        pressed: stage == Some(s),
                        onclick: {
                            let id = st.id.clone();
                            move |_| {
                                if stage != Some(s) {
                                    app.record(Payload::GrowthStage(GrowthStage {
                                        path_version: PATH.version.clone(),
                                        step: id.clone(),
                                        stage: s,
                                    }));
                                }
                            }
                        },
                    }
                }
            }

            div { class: "actions",
                if let Some(n) = next {
                    button {
                        onclick: { let id = n.id.clone(); move |_| app.go(View::GrowthStep(id.clone())) },
                        "Passo successivo: {n.name}"
                    }
                }
                button { class: "quiet", onclick: move |_| app.up(), "Torna al percorso" }
            }
        }
    }
}

// ---- A card ----------------------------------------------------------------

#[component]
pub fn CardView(step: String, card: String, review: bool) -> Element {
    let app = use_app();
    let Some(st) = PATH.step(&step) else {
        return rsx! { p { "Questa scheda non c'è più." } };
    };
    let Some(c) = st.card(&card) else {
        return rsx! { p { "Questa scheda non c'è più." } };
    };
    let pos = st.cards.iter().position(|x| x.id == c.id).unwrap_or(0);
    let next = st.cards.get(pos + 1);
    let already =
        app.events.read().iter().any(
            |e| matches!(&e.payload, Payload::GrowthRead(r) if r.step == st.id && r.card == c.id),
        );

    let mark = move |as_review: bool| {
        app.record(Payload::GrowthRead(GrowthRead {
            path_version: PATH.version.clone(),
            step: st.id.clone(),
            card: c.id.clone(),
            review: as_review,
        }))
        .is_some()
    };

    let go_on = move || match next {
        Some(n) => app.go(View::GrowthCard(st.id.clone(), n.id.clone(), false)),
        None => app.up(),
    };

    rsx! {
        section { class: "fade",
            ScreenHead {
                title: c.title.clone(),
                subtitle: if review { format!("Ripasso · {}", st.name) } else { format!("{} · scheda {} di {}", st.name, pos + 1, st.cards.len()) },
                up: if review { "Crescita".to_string() } else { st.name.clone() },
            }
            p { class: "reading", "{c.text}" }

            div { class: "actions",
                if review {
                    button {
                        class: "primary",
                        onclick: move |_| if mark(true) {
                            app.show_toast("Ripassata. Tornerà fra un po' di tempo.", None);
                            app.up();
                        },
                        "Riletta"
                    }
                    button {
                        onclick: move |_| if mark(true) {
                            app.go(View::GrowthPractice(st.id.clone(), "spiega".into()));
                        },
                        "Riletta: ora la spiego con parole mie"
                    }
                    button {
                        class: "quiet",
                        onclick: move |_| {
                            let today = time::today();
                            app.update_settings(|s| s.growth_review_dismissed_on = Some(today));
                            app.up();
                        },
                        "Non ora"
                    }
                } else if !already {
                    button {
                        class: "primary",
                        onclick: move |_| if mark(false) {
                            if next.is_none() {
                                app.show_toast("Hai letto tutte le schede di questo passo. Se ti va, prova un esercizio.", None);
                            }
                            go_on();
                        },
                        if next.is_some() { "Letta, avanti" } else { "Letta" }
                    }
                    button { class: "quiet", onclick: move |_| app.up(), "Più tardi" }
                } else {
                    p { class: "hint", "L'hai già letta. Tornerà da sola, ogni tanto, come ripasso." }
                    button {
                        class: "primary",
                        onclick: move |_| go_on(),
                        if next.is_some() { "Scheda successiva" } else { "Torna al passo" }
                    }
                }
            }
        }
    }
}

// ---- An exercise -------------------------------------------------------------

#[component]
pub fn PracticeView(step: String, practice: String) -> Element {
    let Some(st) = PATH.step(&step) else {
        return rsx! { p { "Questo esercizio non c'è più." } };
    };
    let Some(pr) = st.practice(&practice) else {
        return rsx! { p { "Questo esercizio non c'è più." } };
    };
    rsx! {
        section { class: "fade",
            ScreenHead { title: pr.title.clone(), icon: "growth", subtitle: st.name.clone(), up: st.name.clone() }
            p { class: "idea", "{pr.intro}" }
            if pr.kind == PracticeKind::Matrix {
                Matrix { step: st.id.clone(), practice: pr.id.clone() }
            } else {
                Written { step: st.id.clone(), practice: pr.id.clone() }
            }
            if pr.kind == PracticeKind::Renewal {
                Renewal { step: st.id.clone(), practice: pr.id.clone() }
            }
        }
    }
}

/// Every kind but the matrix: an optional choice, then text answers.
#[component]
fn Written(step: String, practice: String) -> Element {
    let app = use_app();
    let st = PATH.step(&step).expect("checked by PracticeView");
    let pr = st.practice(&practice).expect("checked by PracticeView");
    let is_mission = pr.kind == PracticeKind::Mission;
    let mut texts = use_signal(|| {
        if is_mission {
            growth::mission(&app.events.peek())
                .map(|(_, p)| p.texts.clone())
                .unwrap_or_default()
        } else {
            BTreeMap::new()
        }
    });
    let mut choice = use_signal(|| None::<String>);
    let filled = choice.read().is_some() || texts.read().values().any(|t| !t.trim().is_empty());

    let save = move |_| {
        let answers: BTreeMap<String, String> = texts
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.trim().to_owned()))
            .filter(|(_, v)| !v.is_empty())
            .collect();
        let payload = Payload::GrowthPractice(GrowthPractice {
            path_version: PATH.version.clone(),
            step: st.id.clone(),
            practice: pr.id.clone(),
            choice: choice.read().clone(),
            ref_id: None,
            texts: answers,
        });
        if app.record(payload).is_some() {
            app.show_toast(
                if is_mission {
                    "Salvata. Anche la versione di prima resta."
                } else {
                    "Fatto. Resta solo qui, sul telefono."
                },
                None,
            );
            app.up();
        }
    };

    let events = app.events.read();
    let earlier: Vec<Event> = practices_of(&events, &st.id, &pr.id)
        .into_iter()
        .cloned()
        .collect();
    drop(events);

    rsx! {
        if !pr.choices.is_empty() {
            div { class: "chips", role: "group", aria_label: "{pr.title}",
                for c in pr.choices.iter() {
                    Chip {
                        key: "{c.id}",
                        label: c.label.clone(),
                        pressed: choice.read().as_deref() == Some(c.id.as_str()),
                        onclick: {
                            let id = c.id.clone();
                            move |_| {
                                let same = choice.read().as_deref() == Some(id.as_str());
                                choice.set(if same { None } else { Some(id.clone()) });
                            }
                        },
                    }
                }
            }
        }
        for q in pr.prompts.iter() {
            div { key: "{q.id}",
                label { r#for: "gp-{q.id}", "{q.label}" }
                textarea {
                    id: "gp-{q.id}",
                    rows: if is_mission || q.id == "spiegazione" { "5" } else { "3" },
                    maxlength: "2000",
                    autocomplete: "off",
                    // Uncontrolled: fast typing never loses letters to a re-render.
                    initial_value: texts.peek().get(&q.id).cloned().unwrap_or_default(),
                    oninput: {
                        let id = q.id.clone();
                        move |e: FormEvent| { texts.write().insert(id.clone(), e.value()); }
                    },
                }
            }
        }
        p { class: "hint", "Ogni campo si può saltare. Quello che scrivi resta solo sul telefono, cifrato, e non esce mai nelle esportazioni per la ricerca." }
        div { class: "actions",
            button { class: "primary", disabled: !filled, onclick: save, if is_mission { "Salva la missione" } else { "Salva" } }
            button { class: "quiet", onclick: move |_| app.up(), "Non ora" }
        }

        if !earlier.is_empty() {
            h2 { class: "sec", Icon { name: "list" } if is_mission { "Le versioni" } else { "Le volte precedenti" } }
            for e in earlier {
                if let Payload::GrowthPractice(p) = &e.payload {
                    div { key: "{e.id}", class: "card earlier",
                        p { class: "muted", "{day_label(&e.ts)}"
                            if let Some(c) = &p.choice { " · {choice_label(pr, c)}" }
                        }
                        for q in pr.prompts.iter() {
                            if let Some(t) = p.texts.get(&q.id) {
                                p { key: "{q.id}", span { class: "label-inline", "{q.label}: " } "{t}" }
                            }
                        }
                        button {
                            class: "quiet small",
                            onclick: { let id = e.id.clone(); move |_| app.go(View::Edit(id.clone())) },
                            "Cambia l'ora o togli"
                        }
                    }
                }
            }
        }
    }
}

/// Renewal per dimension over `days` days up to now, as bar rows.
pub fn renewal_rows(
    events: &[Event],
    links: &BTreeMap<String, Dimension>,
    days: i64,
) -> Vec<Counted> {
    let now = jiff::Timestamp::now();
    let from = now - jiff::SignedDuration::from_hours(24 * days);
    let counts = growth::renewal(
        events,
        links,
        from,
        now + jiff::SignedDuration::from_secs(1),
    );
    dimension_rows(counts)
}

/// The four dimensions with their counts (all four, so balance is visible).
pub fn dimension_rows(counts: [usize; 4]) -> Vec<Counted> {
    Dimension::ALL
        .iter()
        .zip(counts)
        .map(|(d, n)| Counted {
            label: dimension_label(*d),
            times: n,
            fulfilment: None,
        })
        .collect()
}

pub fn dimension_label(d: Dimension) -> String {
    PATH.step("affilare")
        .and_then(|s| s.practice("rinnovamento"))
        .map(|p| choice_label(p, d.id()))
        .unwrap_or_else(|| d.id().to_owned())
}

/// Habit 7: the last week by dimension, and which care gestures count where.
#[component]
fn Renewal(step: String, practice: String) -> Element {
    let app = use_app();
    let links = app.settings.read().care_dimensions.clone();
    let rows = renewal_rows(&app.events.read(), &links, 7);
    let labels = app.lists.read().care_labels.clone();
    let any = rows.iter().any(|r| r.times > 0);

    rsx! {
        h2 { class: "sec", Icon { name: "growth" } "Gli ultimi sette giorni" }
        if any {
            Bars { rows }
        } else {
            p { class: "muted", "Ancora niente in questi giorni. Va bene così." }
        }
        p { class: "hint", "Un'immagine dell'equilibrio, non un obiettivo: nessuna dimensione deve arrivare a un numero." }

        h2 { class: "sec", Icon { name: "care" } "I tuoi gesti di cura" }
        p { class: "muted", "Collega un gesto a una dimensione e conterà qui da solo, ogni volta che lo registri. Tocca di nuovo per scollegarlo." }
        if labels.is_empty() {
            p { class: "hint", "Non hai ancora gesti di cura: li aggiungi in Impostazioni › Le tue liste." }
        }
        for label in labels {
            div { key: "{label}", class: "matrix-row",
                p { "{label}" }
                div { class: "chips", role: "group", aria_label: "Dimensione di {label}",
                    for d in Dimension::ALL {
                        Chip {
                            key: "{d:?}",
                            label: dimension_label(d),
                            pressed: links.get(&label) == Some(&d),
                            onclick: {
                                let label = label.clone();
                                move |_| {
                                    let label = label.clone();
                                    app.update_settings(move |s| {
                                        if s.care_dimensions.get(&label) == Some(&d) {
                                            s.care_dimensions.remove(&label);
                                        } else {
                                            s.care_dimensions.insert(label, d);
                                        }
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// A heading of the matrix: its quadrant (None = not placed yet), title, tasks.
type Group = (Option<Quadrant>, String, Vec<(Event, String)>);

/// Habit 3: the open tasks, each in the quadrant chosen for it.
#[component]
fn Matrix(step: String, practice: String) -> Element {
    let app = use_app();
    let st = PATH.step(&step).expect("checked by PracticeView");
    let pr = st.practice(&practice).expect("checked by PracticeView");
    let events = app.events.read();
    let placed = growth::matrix(&events);
    let mut open: Vec<(Event, String)> = events
        .iter()
        .filter_map(|e| match &e.payload {
            Payload::Task(t) if t.is_open() => Some((e.clone(), t.title.clone())),
            _ => None,
        })
        .collect();
    drop(events);
    open.sort_by(|a, b| a.0.ts.cmp(&b.0.ts));

    let place = move |task: String, q: Option<Quadrant>| {
        app.record(Payload::GrowthPractice(GrowthPractice {
            path_version: PATH.version.clone(),
            step: st.id.clone(),
            practice: pr.id.clone(),
            choice: q.map(|q| q.id().to_owned()),
            ref_id: Some(task),
            texts: BTreeMap::new(),
        }));
    };

    let unplaced: Vec<(Event, String)> = open
        .iter()
        .filter(|(e, _)| !placed.contains_key(&e.id))
        .cloned()
        .collect();
    let mut groups: Vec<Group> = vec![];
    if !unplaced.is_empty() {
        groups.push((None, "Da collocare".into(), unplaced));
    }
    for q in Quadrant::ALL {
        let tasks: Vec<_> = open
            .iter()
            .filter(|(e, _)| placed.get(&e.id) == Some(&q))
            .cloned()
            .collect();
        groups.push((Some(q), choice_label(pr, q.id()), tasks));
    }

    rsx! {
        if open.is_empty() {
            div { class: "card soft",
                p { class: "first", "Non ci sono cose da fare aperte. Quando ne aggiungi qualcuna dalla Home, la ritrovi qui." }
            }
        } else {
            p { class: "hint", "Tocca un numero per mettere una cosa in quel quadrante; toccalo di nuovo per toglierla. Il quadrante II è quello che, col tempo, alleggerisce gli altri." }
            for (q, title, tasks) in groups {
                div { key: "{title}", class: "card group quadrant",
                    h2 { class: "first", "{title}" }
                    if tasks.is_empty() {
                        p { class: "muted", "Per ora niente." }
                    }
                    for (e, name) in tasks {
                        div { key: "{e.id}", class: "matrix-row",
                            p { "{name}" }
                            div { class: "chips", role: "group", aria_label: "Quadrante di: {name}",
                                for target in Quadrant::ALL {
                                    Chip {
                                        key: "{target:?}",
                                        label: ["I", "II", "III", "IV"][target as usize],
                                        pressed: q == Some(target),
                                        onclick: {
                                            let id = e.id.clone();
                                            move |_| place(id.clone(), if q == Some(target) { None } else { Some(target) })
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
