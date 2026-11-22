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
use crate::theme::{self, Look};

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
    Summary,
    Questionnaire(String),
    Data,
    /// Crescita: the path.
    Growth,
    /// A step of the path (step id).
    GrowthStep(String),
    /// A card (step id, card id, opened as a review).
    GrowthCard(String, String, bool),
    /// An exercise (step id, practice id).
    GrowthPractice(String, String),
}

impl View {
    /// Where "back" leads: Home, except inside Crescita.
    pub fn parent(&self) -> View {
        match self {
            Self::GrowthStep(_) => Self::Growth,
            Self::GrowthCard(_, _, true) => Self::Growth,
            Self::GrowthCard(s, _, false) | Self::GrowthPractice(s, _) => {
                Self::GrowthStep(s.clone())
            }
            _ => Self::Home,
        }
    }

    /// How deep the screen is: Home 0, sections 1, inside Crescita up to 3.
    pub fn depth(&self) -> u32 {
        match self {
            Self::Home => 0,
            other => 1 + other.parent().depth(),
        }
    }
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
    /// Theme and day/evening mode: readable before unlocking (see theme.rs).
    pub look: Signal<Look>,
    toast_seq: Signal<u64>,
    last_phrase: Signal<Option<&'static str>>,
    /// An invitation tapped while Kuracare was locked: followed after unlocking.
    tapped: Signal<Option<Tapped>>,
    /// History entries pushed for the back gesture (one per level gone down).
    hist: Signal<u32>,
    /// popstate events caused by our own history.back()/go(), to be ignored.
    own_pops: Signal<u32>,
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
            look: Signal::new(Look::load()),
            toast_seq: Signal::new(0),
            last_phrase: Signal::new(None),
            tapped: Signal::new(None),
            hist: Signal::new(0),
            own_pops: Signal::new(0),
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
    /// every change that can move them (settings, check-ins, tasks, growth reads).
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
            "review" => self.go(View::Growth),
            _ => self.back(),
        }
    }

    pub fn lock(mut self) {
        vault().lock();
        self.events.set(vec![]);
        self.lists.set(Lists::default());
        self.settings.set(Settings::default());
        self.toast.set(None);
        self.back();
        self.phase.set(Phase::Locked);
    }

    // Android's back gesture walks the WebView history. One entry is pushed
    // for every level gone down with a tap (Chrome skips entries pushed
    // without a user gesture, so none is ever pushed from popstate), and one
    // is taken away for every level climbed with our own buttons.

    pub fn go(mut self, v: View) {
        if v.depth() > self.view.peek().depth() {
            document::eval("history.pushState({kuracare: 1}, '')");
            *self.hist.write() += 1;
        }
        self.toast.set(None);
        self.show(v);
    }

    /// One level up (Home for every screen outside Crescita).
    pub fn up(mut self) {
        let parent = self.view.peek().parent();
        if parent == View::Home {
            self.back();
            return;
        }
        if *self.hist.peek() > 0 {
            *self.hist.write() -= 1;
            *self.own_pops.write() += 1;
            document::eval("history.back()");
        }
        self.show(parent);
    }

    /// Straight back to Home, dropping every entry pushed on the way.
    pub fn back(mut self) {
        let n = *self.hist.peek();
        if n > 0 {
            self.hist.set(0);
            *self.own_pops.write() += 1;
            document::eval(&format!("history.go(-{n})"));
        }
        if *self.view.peek() != View::Home {
            self.show(View::Home);
        }
    }

    /// A popstate: ours (ignored), or the back gesture (one level up).
    pub fn popped(mut self) {
        if *self.own_pops.peek() > 0 {
            *self.own_pops.write() -= 1;
            return;
        }
        let n = *self.hist.peek();
        self.hist.set(n.saturating_sub(1));
        let parent = self.view.peek().parent();
        if parent == View::Home {
            self.back();
        } else {
            self.show(parent);
        }
    }

    fn show(mut self, v: View) {
        self.view.set(v);
        document::eval("window.scrollTo(0, 0)");
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
        if matches!(
            e.payload,
            Payload::Checkin(_) | Payload::Task(_) | Payload::GrowthRead(_)
        ) {
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

    pub fn set_look(mut self, l: Look) {
        if let Err(err) = l.save() {
            self.error(err);
        }
        self.look.set(l);
        theme::apply(l);
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

#[cfg(test)]
mod tests {
    use super::View;

    #[test]
    fn crescita_climbs_one_level_at_a_time() {
        let practice = View::GrowthPractice("basi".into(), "spiega".into());
        assert_eq!(practice.parent(), View::GrowthStep("basi".into()));
        assert_eq!(practice.depth(), 3);
        let review = View::GrowthCard("basi".into(), "paradigmi".into(), true);
        assert_eq!(review.parent(), View::Growth);
        assert_eq!(review.depth(), 2);
        assert_eq!(View::GrowthStep("basi".into()).parent(), View::Growth);
        assert_eq!(View::Growth.parent(), View::Home);
        assert_eq!(View::Settings.depth(), 1);
        assert_eq!(View::Home.depth(), 0);
    }
}
