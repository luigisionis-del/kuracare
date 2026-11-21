//! Kuracare app (Dioxus). All logic and data live in `kuracare-core`.

mod labels;
mod photo;
mod platform;
mod state;
mod views;

use std::time::Duration;

use dioxus::prelude::*;

use state::{AppState, Phase, View, idle_seconds, touch};
use views::parts::ToastBar;

const CSS: &str = include_str!("../assets/app.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app = use_context_provider(AppState::new);

    // Idle lock: checked every 10 s, and on resume (the loop runs again then).
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if *app.phase.peek() == Phase::Open
                && idle_seconds() >= i64::from(app.settings.peek().lock_after_minutes) * 60
            {
                app.lock();
            }
        }
    });

    // Android back gesture: sub-views push a history entry; going back shows Home.
    use_future(move || async move {
        let mut ev = document::eval(
            r#"window.addEventListener('popstate', () => dioxus.send('back'));
               await new Promise(() => {});"#,
        );
        while ev.recv::<String>().await.is_ok() {
            let mut view = app.view;
            view.set(View::Home);
        }
    });

    let phase = app.phase.read().clone();
    let view = app.view.read().clone();
    rsx! {
        style { {CSS} }
        main {
            onpointerdown: move |_| touch(),
            onkeydown: move |_| touch(),
            match phase {
                Phase::Onboarding => rsx! { views::onboarding::Onboarding {} },
                Phase::Locked => rsx! { views::unlock::Unlock {} },
                Phase::Open => rsx! {
                    match view {
                        View::Home => rsx! { views::home::Home {} },
                        View::Checkin => rsx! { views::checkin::Checkin {} },
                        View::Thought => rsx! { views::thought::Thought {} },
                        View::Meal(t) => rsx! { views::meal::Meal { meal_type: t } },
                        View::Edit(id) => rsx! { views::edit::Edit { key: "{id}", id: id.clone() } },
                        View::Enrich => rsx! { views::enrich::Enrich {} },
                        View::Lists => rsx! { views::lists::Lists {} },
                    }
                    ToastBar {}
                },
            }
        }
    }
}
