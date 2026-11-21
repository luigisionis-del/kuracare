use dioxus::prelude::*;
use kuracare_core::kinds::{Route, positive};
use kuracare_core::lists::{
    Lists as ListsData, StrainGroup, StrainPreset, SupplementPreset, new_id,
};

use crate::labels::{num, route_label};
use crate::state::use_app;
use crate::views::parts::{Chip, Icon, ScreenHead};

/// Strain being edited: numbers as text, so "0,5" works with an Italian keyboard.
#[derive(Clone, PartialEq)]
struct StrainEdit {
    p: StrainPreset,
    thc: String,
    cbd: String,
    amount: String,
}

#[derive(Clone, PartialEq)]
struct SuppEdit {
    p: SupplementPreset,
    amount: String,
}

#[component]
pub fn Lists() -> Element {
    let app = use_app();
    let start = app.lists.read().clone();
    let mut strains = use_signal(|| {
        start
            .strains
            .iter()
            .map(|s| StrainEdit {
                thc: num(s.thc_pct),
                cbd: num(s.cbd_pct),
                amount: num(s.amount),
                p: s.clone(),
            })
            .collect::<Vec<_>>()
    });
    let mut supps = use_signal(|| {
        start
            .supplements
            .iter()
            .map(|s| SuppEdit {
                amount: num(s.amount),
                p: s.clone(),
            })
            .collect::<Vec<_>>()
    });
    let care = use_signal(|| start.care_labels.clone());
    let thoughts = use_signal(|| start.thought_labels.clone());
    let rooms = use_signal(|| start.rooms.clone());
    let activities = use_signal(|| start.home_activities.clone());

    let mut add_strain = move |group: StrainGroup| {
        strains.write().push(StrainEdit {
            p: StrainPreset {
                id: new_id("s"),
                name: String::new(),
                group,
                thc_pct: None,
                cbd_pct: None,
                amount: None,
                unit: "g".into(),
                route: None,
            },
            thc: String::new(),
            cbd: String::new(),
            amount: String::new(),
        });
    };

    let save = move |_| {
        let lists = ListsData {
            version: 1,
            strains: strains
                .read()
                .iter()
                .filter(|s| !s.p.name.trim().is_empty())
                .map(|s| StrainPreset {
                    name: s.p.name.trim().into(),
                    thc_pct: positive(&s.thc, 100.0),
                    cbd_pct: positive(&s.cbd, 100.0),
                    amount: positive(&s.amount, f64::INFINITY),
                    ..s.p.clone()
                })
                .collect(),
            supplements: supps
                .read()
                .iter()
                .filter(|s| !s.p.name.trim().is_empty())
                .map(|s| SupplementPreset {
                    name: s.p.name.trim().into(),
                    amount: positive(&s.amount, f64::INFINITY),
                    ..s.p.clone()
                })
                .collect(),
            care_labels: care.read().clone(),
            thought_labels: thoughts.read().clone(),
            rooms: rooms.read().clone(),
            home_activities: activities.read().clone(),
        };
        app.save_lists(lists);
        app.show_toast("Liste salvate.", None);
        app.back();
    };

    rsx! {
        section { class: "fade",
            ScreenHead {
                title: "Le tue liste",
                icon: "list",
                subtitle: "Sono i bottoni da un tocco. I valori li scegli tu: Kuracare non suggerisce varietà né quantità.",
            }

            h2 { class: "sec", Icon { name: "leaf" } "Varietà" }
            for (i, s) in strains.read().iter().cloned().enumerate() {
                div { key: "{s.p.id}", class: "card", role: "group", aria_label: if s.p.name.is_empty() { "Nuova varietà".to_string() } else { s.p.name.clone() },
                    label { r#for: "sn{i}", "Nome" }
                    input { id: "sn{i}", r#type: "text", value: "{s.p.name}", oninput: move |e| strains.write()[i].p.name = e.value() }
                    div { class: "chips", role: "group", aria_label: "Prevalenza",
                        Chip { label: "Prevalenza THC", pressed: s.p.group == StrainGroup::Thc, onclick: move |_| strains.write()[i].p.group = StrainGroup::Thc }
                        Chip { label: "Prevalenza CBD", pressed: s.p.group == StrainGroup::Cbd, onclick: move |_| strains.write()[i].p.group = StrainGroup::Cbd }
                    }
                    div { class: "grid2",
                        div { label { r#for: "st{i}", "THC %" } input { id: "st{i}", r#type: "text", inputmode: "decimal", value: "{s.thc}", oninput: move |e| strains.write()[i].thc = e.value() } }
                        div { label { r#for: "sc{i}", "CBD %" } input { id: "sc{i}", r#type: "text", inputmode: "decimal", value: "{s.cbd}", oninput: move |e| strains.write()[i].cbd = e.value() } }
                        div { label { r#for: "sa{i}", "Quantità abituale" } input { id: "sa{i}", r#type: "text", inputmode: "decimal", value: "{s.amount}", oninput: move |e| strains.write()[i].amount = e.value() } }
                        div { label { r#for: "su{i}", "Unità" } input { id: "su{i}", r#type: "text", value: "{s.p.unit}", oninput: move |e| strains.write()[i].p.unit = e.value() } }
                    }
                    p { class: "label", "Via abituale" }
                    div { class: "chips",
                        for r in Route::ALL {
                            Chip {
                                key: "{r:?}",
                                label: route_label(r),
                                pressed: s.p.route == Some(r),
                                onclick: move |_| {
                                    let mut w = strains.write();
                                    w[i].p.route = if w[i].p.route == Some(r) { None } else { Some(r) };
                                },
                            }
                        }
                    }
                    button { class: "quiet small", onclick: move |_| { strains.write().remove(i); }, "Togli dalla lista" }
                }
            }
            div { class: "row",
                button { class: "small", onclick: move |_| add_strain(StrainGroup::Thc), "+ Varietà THC" }
                button { class: "small", onclick: move |_| add_strain(StrainGroup::Cbd), "+ Varietà CBD" }
            }

            h2 { class: "sec", Icon { name: "leaf" } "Integratori" }
            for (i, s) in supps.read().iter().cloned().enumerate() {
                div { key: "{s.p.id}", class: "card", role: "group", aria_label: if s.p.name.is_empty() { "Nuovo integratore".to_string() } else { s.p.name.clone() },
                    label { r#for: "in{i}", "Nome" }
                    input { id: "in{i}", r#type: "text", value: "{s.p.name}", oninput: move |e| supps.write()[i].p.name = e.value() }
                    div { class: "grid2",
                        div { label { r#for: "ia{i}", "Quantità abituale" } input { id: "ia{i}", r#type: "text", inputmode: "decimal", value: "{s.amount}", oninput: move |e| supps.write()[i].amount = e.value() } }
                        div { label { r#for: "iu{i}", "Unità" } input { id: "iu{i}", r#type: "text", value: "{s.p.unit}", oninput: move |e| supps.write()[i].p.unit = e.value() } }
                    }
                    button { class: "quiet small", onclick: move |_| { supps.write().remove(i); }, "Togli dalla lista" }
                }
            }
            button {
                class: "small",
                onclick: move |_| supps.write().push(SuppEdit { p: SupplementPreset { id: new_id("i"), name: String::new(), amount: None, unit: "mg".into() }, amount: String::new() }),
                "+ Integratore"
            }

            LabelList { title: "Gesti di cura", icon: "care", placeholder: "Un nuovo gesto", items: care }
            LabelList { title: "Stanze", icon: "home", placeholder: "Una nuova stanza", items: rooms }
            LabelList { title: "Cose da fare in casa", icon: "home", placeholder: "Una nuova attività", items: activities }
            LabelList { title: "Pensieri che tornano", icon: "thought", placeholder: "Una nuova etichetta", items: thoughts }

            div { class: "actions dock",
                button { class: "primary", onclick: save, "Salva le liste" }
                button { class: "quiet", onclick: move |_| app.back(), "Lascia com'erano" }
            }
        }
    }
}

#[component]
fn LabelList(
    title: String,
    icon: &'static str,
    placeholder: String,
    items: Signal<Vec<String>>,
) -> Element {
    let mut items = items;
    let mut fresh = use_signal(String::new);
    let add = move |e: FormEvent| {
        e.prevent_default();
        let t = fresh.read().trim().to_owned();
        if !t.is_empty() && !items.read().contains(&t) {
            items.write().push(t);
            fresh.set(String::new());
        }
    };
    rsx! {
        h2 { class: "sec", Icon { name: icon } "{title}" }
        div { class: "chips",
            for (i, l) in items.read().iter().cloned().enumerate() {
                button {
                    key: "{l}", class: "chip removable", aria_label: "Togli {l}",
                    onclick: move |_| { items.write().remove(i); },
                    "{l}"
                    Icon { name: "close" }
                }
            }
        }
        form { class: "row add", onsubmit: add,
            input { r#type: "text", aria_label: "{placeholder}", placeholder: "{placeholder}", value: "{fresh}", oninput: move |e| fresh.set(e.value()) }
            button { class: "small", r#type: "submit", "Aggiungi" }
        }
    }
}
