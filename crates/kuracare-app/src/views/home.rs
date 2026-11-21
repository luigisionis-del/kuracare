use dioxus::prelude::*;
use kuracare_core::kinds::{
    Cannabis, Care, Home as HomeCare, MealType, Payload, Reflection, Supplement, Task, TaskStatus,
};
use kuracare_core::lists::{StrainGroup, StrainPreset, SupplementPreset};
use kuracare_core::time;
use kuracare_core::{Event, Origin};

use crate::labels::{day_label, describe, greeting, long_date, meal_label, pause_label};
use crate::state::{View, use_app};
use crate::views::parts::Icon;

#[component]
pub fn Home() -> Element {
    let app = use_app();
    let mut reflection_text = use_signal(String::new);
    let mut task_text = use_signal(String::new);
    // The chip just tapped blooms in saffron for a moment.
    let mut bloom = use_signal(|| None::<String>);
    let mut bloom_seq = use_signal(|| 0u64);
    let mut flourish = move |key: String| {
        let seq = *bloom_seq.peek() + 1;
        bloom_seq.set(seq);
        bloom.set(Some(key));
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1400)).await;
            if *bloom_seq.peek() == seq {
                bloom.set(None);
            }
        });
    };

    let now = time::now();
    let today = time::today();
    let evening = now.hour() >= 18;
    let settings = app.settings.read().clone();
    let paused = settings.is_paused(now.timestamp());
    let lists = app.lists.read().clone();
    let todays: Vec<_> = app
        .events
        .read()
        .iter()
        .filter(|e| time::local_day(&e.ts) == today && !matches!(e.payload, Payload::Task(_)))
        .cloned()
        .collect();
    // Open tasks: due today or before, any time, later (each sorted by when).
    let mut open: Vec<(Event, Task)> = app
        .events
        .read()
        .iter()
        .filter_map(|e| match &e.payload {
            Payload::Task(t) if t.is_open() => Some((e.clone(), t.clone())),
            _ => None,
        })
        .collect();
    open.sort_by(|(a, x), (b, y)| {
        (&x.due_date, &x.due_time, &a.ts).cmp(&(&y.due_date, &y.due_time, &b.ts))
    });
    let (dated, anytime): (Vec<_>, Vec<_>) =
        open.into_iter().partition(|(_, t)| t.due_date.is_some());
    let (for_today, later): (Vec<_>, Vec<_>) = dated
        .into_iter()
        .partition(|(_, t)| t.due_date.as_deref() <= Some(today.as_str()));
    let reflection = todays
        .iter()
        .find(|e| matches!(e.payload, Payload::Reflection(_)))
        .cloned();
    let meals_left: Vec<MealType> = [MealType::Breakfast, MealType::Lunch, MealType::Dinner]
        .into_iter()
        .filter(|t| {
            !todays
                .iter()
                .any(|e| matches!(&e.payload, Payload::Meal(m) if m.meal_type == *t))
        })
        .collect();
    let show_meal_recall = evening
        && !paused
        && !meals_left.is_empty()
        && settings.meal_recall_dismissed_on.as_deref() != Some(today.as_str());
    let thc: Vec<StrainPreset> = lists
        .strains
        .iter()
        .filter(|s| s.group == StrainGroup::Thc)
        .cloned()
        .collect();
    let cbd: Vec<StrainPreset> = lists
        .strains
        .iter()
        .filter(|s| s.group == StrainGroup::Cbd)
        .cloned()
        .collect();

    let mut care = move |label: String| {
        flourish(format!("care:{label}"));
        if let Some(e) = app.record(Payload::Care(Care {
            label,
            duration_min: None,
            fulfilment: None,
        })) {
            app.show_toast(
                app.recognition(),
                Some(("Quanto ti ha fatto bene?".into(), e.id)),
            );
        }
    };
    let mut room = move |room: String| {
        flourish(format!("room:{room}"));
        if let Some(e) = app.record(Payload::Home(HomeCare {
            room,
            activity: None,
            duration_min: None,
            fulfilment: None,
        })) {
            app.show_toast(app.recognition(), Some(("Aggiungi dettagli".into(), e.id)));
        }
    };
    let strain = move |s: StrainPreset| {
        let payload = Payload::Cannabis(Cannabis {
            preset_id: Some(s.id),
            strain: s.name.clone(),
            thc_pct: s.thc_pct,
            cbd_pct: s.cbd_pct,
            amount: s.amount,
            unit: s.unit,
            route: s.route,
        });
        if let Some(e) = app.record(payload) {
            app.show_toast(
                format!("{}: segnato alle {}.", s.name, time::hhmm(&e.ts)),
                Some(("Modifica".into(), e.id)),
            );
        }
    };
    let supplement = move |s: SupplementPreset| {
        let payload = Payload::Supplement(Supplement {
            preset_id: Some(s.id),
            name: s.name.clone(),
            amount: s.amount,
            unit: s.unit,
        });
        if let Some(e) = app.record(payload) {
            app.show_toast(
                format!("{}: segnato alle {}.", s.name, time::hhmm(&e.ts)),
                Some(("Modifica".into(), e.id)),
            );
        }
    };
    let add_task = move |e: FormEvent| {
        e.prevent_default();
        let title = task_text.read().trim().to_owned();
        if title.is_empty() {
            return;
        }
        if let Some(e) = app.record(Payload::Task(Task::new(title))) {
            task_text.set(String::new());
            app.show_toast("Segnata.", Some(("Per quando?".into(), e.id)));
        }
    };
    let done = move |mut e: Event| {
        if let Payload::Task(t) = &mut e.payload {
            t.status = TaskStatus::Done;
            t.closed_at = Some(time::now_iso());
        }
        app.change(&e);
        app.show_toast(app.recognition(), None);
    };
    let save_reflection = move |e: FormEvent| {
        e.prevent_default();
        let text = reflection_text.read().trim().to_owned();
        if text.is_empty() {
            return;
        }
        if app
            .record(Payload::Reflection(Reflection { text }))
            .is_some()
        {
            reflection_text.set(String::new());
            app.show_toast(app.recognition(), None);
        }
    };

    let bloomed = bloom.read().clone();
    let chip_class = move |key: String| {
        if bloomed.as_deref() == Some(key.as_str()) {
            "chip bloom"
        } else {
            "chip"
        }
    };

    let reflection_card = rsx! {
        div { class: "card soft",
            h2 { class: "first sec", Icon { name: "evening" } "Cosa ti ha fatto bene oggi?" }
            if let Some(r) = reflection {
                if let Payload::Reflection(p) = &r.payload { p { "{p.text}" } }
                button { class: "quiet small", onclick: move |_| app.go(View::Edit(r.id.clone())), "Cambia" }
            } else {
                form { onsubmit: save_reflection,
                    input {
                        r#type: "text",
                        maxlength: "200",
                        aria_label: "Cosa ti ha fatto bene oggi",
                        placeholder: "Una riga, se ti va",
                        value: "{reflection_text}",
                        oninput: move |e| reflection_text.set(e.value()),
                    }
                    p { class: "hint", "Resta solo su questo telefono." }
                    button { class: "small", r#type: "submit", disabled: reflection_text.read().trim().is_empty(), "Tienilo" }
                }
            }
        }
    };

    rsx! {
        section { class: "fade",
            p { class: "date", {long_date(&now)} }
            h1 { class: "greet", {greeting(now.hour())} }

            // In the evening, what did good comes first.
            if evening { {reflection_card.clone()} }

            div { class: "card",
                h2 { class: "first sec", Icon { name: "checkin" } "Come stai?" }
                if paused {
                    p { class: "muted", "Sei in pausa {pause_label(settings.pause_until.as_deref())}: Kuracare non ti chiede niente. Se ti va, il check-in c'è comunque." }
                    button { class: "wide", onclick: move |_| app.go(View::Checkin(Origin::Spontaneous)), "Fai il check-in" }
                } else {
                    p { class: "muted", "Cinque domande veloci, tutte facoltative." }
                    button { class: "primary wide", onclick: move |_| app.go(View::Checkin(Origin::Spontaneous)), "Fai il check-in" }
                }
            }

            h2 { class: "sec", Icon { name: "tasks" } "Cose da fare" }
            form { class: "row add", onsubmit: add_task,
                input {
                    r#type: "text",
                    maxlength: "140",
                    aria_label: "Una cosa da fare",
                    placeholder: "Una cosa da fare",
                    value: "{task_text}",
                    oninput: move |e| task_text.set(e.value()),
                }
                button { class: "small", r#type: "submit", disabled: task_text.read().trim().is_empty(), "Aggiungi" }
            }
            for (label, group) in [("Per oggi", for_today), ("Quando vuoi", anytime), ("Più avanti", later)] {
                if !group.is_empty() {
                    p { key: "{label}", class: "label", "{label}" }
                    ul { class: "tasks",
                        for (e, t) in group {
                            li { key: "{e.id}",
                                button {
                                    class: "tick",
                                    aria_label: "Fatta: {t.title}",
                                    onclick: { let e = e.clone(); move |_| done(e.clone()) },
                                    span { svg { view_box: "0 0 24 24", "aria-hidden": "true", path { d: "M5 12.5l4.2 4.2L19 7" } } }
                                }
                                button {
                                    class: "entry",
                                    onclick: { let id = e.id.clone(); move |_| app.go(View::Edit(id.clone())) },
                                    if label == "Più avanti" {
                                        if let Some(d) = &t.due_date { span { class: "time", {day_label(d)} } }
                                    } else if let Some(h) = &t.due_time {
                                        span { class: "time", "{h}" }
                                    }
                                    "{t.title}"
                                }
                            }
                        }
                    }
                }
            }

            h2 { class: "sec", Icon { name: "care" } "Un gesto di cura" }
            p { class: "muted", "Un tocco per segnare qualcosa di buono che hai fatto per te." }
            div { class: "chips",
                for l in lists.care_labels.iter().cloned() {
                    button { key: "{l}", class: chip_class(format!("care:{l}")), onclick: move |_| care(l.clone()), "{l}" }
                }
            }

            h2 { class: "sec", Icon { name: "home" } "La casa" }
            p { class: "muted", "Anche sistemare casa è prendersi cura di sé." }
            div { class: "chips",
                for r in lists.rooms.iter().cloned() {
                    button { key: "{r}", class: chip_class(format!("room:{r}")), onclick: move |_| room(r.clone()), "{r}" }
                }
            }

            h2 { class: "sec", Icon { name: "leaf" } "Assunzioni" }
            if !thc.is_empty() {
                p { class: "label", "Prevalenza THC" }
                div { class: "chips",
                    for s in thc {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| strain(s.clone()) }, "{s.name}" }
                    }
                }
            }
            if !cbd.is_empty() {
                p { class: "label", "Prevalenza CBD" }
                div { class: "chips",
                    for s in cbd {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| strain(s.clone()) }, "{s.name}" }
                    }
                }
            }
            if !lists.supplements.is_empty() {
                p { class: "label", "Integratori" }
                div { class: "chips",
                    for s in lists.supplements.iter().cloned() {
                        button { key: "{s.id}", class: "chip", onclick: { let s = s.clone(); move |_| supplement(s.clone()) }, "{s.name}" }
                    }
                }
            }

            h2 { class: "sec", Icon { name: "meal" } "Pasti" }
            div { class: "chips",
                for t in MealType::ALL {
                    button { key: "{meal_label(t)}", class: "chip", onclick: move |_| app.go(View::Meal(Some(t))), {meal_label(t)} }
                }
            }

            if show_meal_recall {
                div { class: "card soft",
                    p { "Ti va di segnare cosa hai mangiato oggi?" }
                    div { class: "chips",
                        for t in meals_left {
                            button { key: "{meal_label(t)}", class: "chip", onclick: move |_| app.go(View::Meal(Some(t))), {meal_label(t)} }
                        }
                    }
                    button {
                        class: "quiet small",
                        onclick: move |_| {
                            let today = time::today();
                            app.update_settings(|s| s.meal_recall_dismissed_on = Some(today));
                        },
                        "Non ora"
                    }
                }
            }

            h2 { class: "sec", Icon { name: "thought" } "Un pensiero che torna" }
            p { class: "muted", "Se c'è un pensiero che gira, puoi posarlo qui." }
            button { onclick: move |_| app.go(View::Thought), "Segna un pensiero" }

            if !evening { {reflection_card} }

            if !todays.is_empty() {
                h2 { class: "sec", Icon { name: "today" } "Oggi" }
                ul { class: "today",
                    for e in todays {
                        li { key: "{e.id}",
                            button {
                                class: "entry",
                                onclick: { let id = e.id.clone(); move |_| app.go(View::Edit(id.clone())) },
                                span { class: "time", {time::hhmm(&e.ts)} }
                                " {describe(&e)}"
                            }
                        }
                    }
                }
            }

            div { class: "actions",
                button { onclick: move |_| app.go(View::Enrich), Icon { name: "sparkle" } " Se ti va: un momento per sistemare" }
                button { onclick: move |_| app.go(View::Settings), "Impostazioni" }
                button { class: "quiet", onclick: move |_| app.lock(), "Chiudi Kuracare" }
            }
        }
    }
}
