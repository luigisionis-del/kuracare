//! Kuracare app (Dioxus). All logic and data live in `kuracare-core`.

mod labels;
mod notify;
mod photo;
mod platform;
mod state;
mod theme;
mod views;

use std::time::Duration;

use dioxus::prelude::*;

use state::{AppState, Phase, View, idle_seconds, touch};
use views::parts::ToastBar;

const CSS: &str = include_str!("../assets/app.css");

/// Content Security Policy of the page inside the WebView. Dioxus talks to the
/// page over a websocket on 127.0.0.1 (that is why the APK needs the INTERNET
/// permission): every other destination is refused. 'unsafe-eval' and
/// 'unsafe-inline' are required by the Dioxus interpreter and by `<style>`.
const CSP: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data: blob:; \
    font-src 'self' data:; \
    connect-src 'self' ws://127.0.0.1:* http://127.0.0.1:*; \
    media-src 'none'; object-src 'none'; frame-src 'none'; worker-src 'none'; \
    form-action 'none'; base-uri 'none'";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app = use_context_provider(AppState::new);
    use_hook(platform::clear_capture_dir);

    // Theme: applied now, whenever it changes, and again every minute so that
    // "Le stagioni" turns on the 1st of the month without a restart.
    let look = app.look;
    use_effect(move || theme::apply(look()));
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            theme::apply(*look.peek());
        }
    });

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

    // An invitation tapped in the notification shade (checked every second).
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Some(t) = notify::take_tapped() {
                app.tapped(t);
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
        document::Meta { http_equiv: "Content-Security-Policy", content: CSP }
        style { {theme::fonts_css()} }
        style { {theme::themes_css()} {CSS} }
        main {
            onpointerdown: move |_| touch(),
            onkeydown: move |_| touch(),
            match phase {
                Phase::Onboarding => rsx! { views::onboarding::Onboarding {} },
                Phase::Locked => rsx! { views::unlock::Unlock {} },
                Phase::Open => rsx! {
                    match view {
                        View::Home => rsx! { views::home::Home {} },
                        View::Checkin(origin) => rsx! { views::checkin::Checkin { origin } },
                        View::Thought => rsx! { views::thought::Thought {} },
                        View::Meal(t) => rsx! { views::meal::Meal { meal_type: t } },
                        View::Edit(id) => rsx! { views::edit::Edit { key: "{id}", id: id.clone() } },
                        View::Enrich => rsx! { views::enrich::Enrich {} },
                        View::Lists => rsx! { views::lists::Lists {} },
                        View::Settings => rsx! { views::settings::Settings {} },
                        View::Summary => rsx! { views::summary::Summary {} },
                        View::Questionnaire(id) => rsx! { views::questionnaire::Questionnaire { key: "{id}", id: id.clone() } },
                    }
                    ToastBar {}
                },
            }
        }
    }
}
