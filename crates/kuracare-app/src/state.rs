//! App-wide state, shared through the Dioxus context. All personal data in
//! memory is dropped on lock; the vault wipes its key.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use dioxus::prelude::*;
use kuracare_core::lists::Lists;
use kuracare_core::settings::Settings;
use kuracare_core::{Event, Origin, Payload, lists, recognition, reminders, repo, settings};

use crate::notify::{self, Tapped};
use crate::platform::vault;

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Onboarding,
    Locked,
    Open,
}

#[derive(Clone, Debug, PartialEq)]
pub enum View {
    Home,
    Checkin(Origin),
    Thought,
    Meal(Option<kuracare_core::kinds::MealType>),
    Edit(String),
    Enrich,
    Lists,
    Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub text: String,
    /// (button label, event id to open)
    pub action: Option<(String, String)>,
}

/// Unix seconds of the last user activity, for the idle lock.
static LAST_ACTIVITY: AtomicI64 = AtomicI64::new(0);

pub fn touch() {
    LAST_ACTIVITY.store(jiff::Timestamp::now().as_second(), Ordering::Relaxed);
}

pub fn idle_seconds() -> i64 {
    jiff::Timestamp::now().as_second() - LAST_ACTIVITY.load(Ordering::Relaxed)
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub phase: Signal<Phase>,
    pub view: Signal<View>,
    pub events: Signal<Vec<Event>>,
    pub lists: Signal<Lists>,
    pub settings: Signal<Settings>,
    pub toast: Signal<Option<Toast>>,
    toast_seq: Signal<u64>,
    last_phrase: Signal<Option<&'static str>>,
    /// An invitation tapped while Kuracare was locked: followed after unlocking.
    tapped: Signal<Option<Tapped>>,
}

pub fn use_app() -> AppState {
    use_context::<AppState>()
}

impl AppState {
    /// Must be called inside a component (creates signals).
    pub fn new() -> Self {
        let v = vault();
        // A relaunch in the same process must never find the vault open.
        v.lock();
        let phase = if v.exists().unwrap_or(false) {
            Phase::Locked
        } else {
            Phase::Onboarding
        };
        Self {
            phase: Signal::new(phase),
            view: Signal::new(View::Home),
            events: Signal::new(vec![]),
            lists: Signal::new(Lists::default()),
            settings: Signal::new(Settings::default()),
            toast: Signal::new(None),
            toast_seq: Signal::new(0),
            last_phrase: Signal::new(None),
            tapped: Signal::new(None),
        }
    }

    /// Loads everything after a successful create/unlock (called on the UI thread).
    pub fn opened(mut self) -> kuracare_core::Result<()> {
        let v = vault();
        let mut s = settings::load(&v)?;
        if s.invite_salt == 0 {
            s.invite_salt = reminders::new_salt();
            settings::save(&v, &s)?;
        }
        self.settings.set(s);
        self.lists.set(lists::load(&v)?);
        self.events.set(repo::load(&v)?);
        self.view.set(View::Home);
        self.phase.set(Phase::Open);
        touch();
        self.sync_invitations();
        if let Some(t) = self.tapped.take() {
            self.follow(t);
        }
        Ok(())
    }

    /// Recomputes the invitations and hands them to the phone. Called after
    /// every change that can move them (settings, check-ins, tasks).
    pub fn sync_invitations(self) {
        let plan = reminders::plan(
            &self.settings.peek(),
            &self.events.peek(),
            &kuracare_core::time::now(),
        );
        notify::set_plan(&plan.to_json());
    }

    /// The user tapped an invitation.
    pub fn tapped(mut self, t: Tapped) {
        if *self.phase.peek() == Phase::Open {
            self.follow(t);
        } else {
            self.tapped.set(Some(t));
        }
    }

    fn follow(self, t: Tapped) {
        if t.is_test() {
            return;
        }
        match t.kind.as_str() {
            "checkin" => self.go(View::Checkin(Origin::Prompted)),
            _ => self.back(),
        }
    }

    pub fn lock(mut self) {
        vault().lock();
        self.events.set(vec![]);
        self.lists.set(Lists::default());
        self.settings.set(Settings::default());
        self.toast.set(None);
        self.view.set(View::Home);
        self.phase.set(Phase::Locked);
    }

    pub fn go(mut self, v: View) {
        if *self.view.peek() == View::Home && v != View::Home {
            // So that Android's back gesture returns home instead of closing the app.
            document::eval("history.pushState({kuracare: 1}, '')");
        }
        self.toast.set(None);
        self.view.set(v);
        document::eval("window.scrollTo(0, 0)");
    }

    pub fn back(mut self) {
        if *self.view.peek() != View::Home {
            self.view.set(View::Home);
            document::eval("if (history.state && history.state.kuracare) history.back()");
            document::eval("window.scrollTo(0, 0)");
        }
    }

    pub fn record(self, payload: Payload) -> Option<Event> {
        self.record_as(payload, Origin::Spontaneous)
    }

    pub fn record_as(mut self, payload: Payload, origin: Origin) -> Option<Event> {
        let e = Event::new(payload, origin);
        match repo::save(&vault(), &e) {
            Ok(()) => {
                {
                    let mut list = self.events.write();
                    list.push(e.clone());
                    repo::sort_newest_first(&mut list);
                }
                self.sync_if_relevant(&e);
                Some(e)
            }
            Err(err) => {
                self.error(err);
                None
            }
        }
    }

    pub fn change(mut self, e: &Event) {
        match repo::update(&vault(), e) {
            Ok(saved) => {
                {
                    let mut list = self.events.write();
                    list.retain(|x| x.id != saved.id);
                    list.push(saved);
                    repo::sort_newest_first(&mut list);
                }
                self.sync_if_relevant(e);
            }
            Err(err) => self.error(err),
        }
    }

    fn sync_if_relevant(self, e: &Event) {
        if matches!(e.payload, Payload::Checkin(_) | Payload::Task(_)) {
            self.sync_invitations();
        }
    }

    pub fn forget(mut self, e: &Event) {
        match repo::remove(&vault(), e) {
            Ok(()) => {
                self.events.write().retain(|x| x.id != e.id);
                self.sync_if_relevant(e);
            }
            Err(err) => self.error(err),
        }
    }

    pub fn find(self, id: &str) -> Option<Event> {
        self.events.peek().iter().find(|e| e.id == id).cloned()
    }

    pub fn save_lists(mut self, l: Lists) {
        match lists::save(&vault(), &l) {
            Ok(()) => self.lists.set(l),
            Err(err) => self.error(err),
        }
    }

    pub fn update_settings(mut self, f: impl FnOnce(&mut Settings)) {
        let mut s = self.settings.peek().clone();
        f(&mut s);
        match settings::save(&vault(), &s) {
            Ok(()) => {
                self.settings.set(s);
                self.sync_invitations();
            }
            Err(err) => self.error(err),
        }
    }

    pub fn show_toast(mut self, text: impl Into<String>, action: Option<(String, String)>) {
        let seq = *self.toast_seq.peek() + 1;
        self.toast_seq.set(seq);
        let secs = if action.is_some() { 8 } else { 5 };
        self.toast.set(Some(Toast {
            text: text.into(),
            action,
        }));
        let mut toast = self.toast;
        let toast_seq = self.toast_seq;
        spawn(async move {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            if *toast_seq.peek() == seq {
                toast.set(None);
            }
        });
    }

    /// A sober phrase from content/riconoscimenti.it.txt, never the same twice in a row.
    pub fn recognition(mut self) -> &'static str {
        let p = recognition::random(*self.last_phrase.peek());
        self.last_phrase.set(Some(p));
        p
    }

    pub fn error(self, err: impl std::fmt::Display) {
        self.show_toast(format!("Qualcosa non è andato: {err}"), None);
    }
}
