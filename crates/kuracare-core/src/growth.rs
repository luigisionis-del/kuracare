//! "Crescita": a guided path through the ideas of Stephen R. Covey's
//! *The 7 Habits of Highly Effective People* (1989; Italian edition
//! *Le 7 regole per avere successo*).
//!
//! The texts live in `content/crescita/percorso.it.json`, written in Kuracare's
//! own words (no quotations from the book). Luigi may edit them; ids stay stable.
//!
//! Mastery without scores: what is recorded is what was read, what was
//! practised and, if wanted, a self-assessment in words. The path is guided
//! (a suggested next step) but never locked. Reviews come back on expanding
//! intervals, as invitations only.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::event::Event;
use crate::kinds::Payload;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub author: String,
    pub title: String,
    pub italian_title: String,
    pub year: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PracticeKind {
    /// Free text answers to the prompts.
    Write,
    /// One choice, plus optional text.
    Choose,
    /// Like `Write`; the latest one is the current mission, older ones stay.
    Mission,
    /// One quadrant per task (`refId` = task event id).
    Matrix,
    /// One of the four dimensions of renewal.
    Renewal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Practice {
    pub id: String,
    pub title: String,
    pub kind: PracticeKind,
    pub intro: String,
    pub prompts: Vec<Prompt>,
    #[serde(default)]
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub number: u8,
    pub name: String,
    pub tagline: String,
    pub idea: String,
    pub cards: Vec<Card>,
    pub practices: Vec<Practice>,
}

impl Step {
    pub fn card(&self, id: &str) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }
    pub fn practice(&self, id: &str) -> Option<&Practice> {
        self.practices.iter().find(|p| p.id == id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Path {
    /// Content version, stored with every record and never migrated.
    pub version: String,
    pub source: Source,
    pub steps: Vec<Step>,
}

impl Path {
    pub fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }
}

const BUILTIN: &str = include_str!("../../../content/crescita/percorso.it.json");

pub fn path() -> Path {
    serde_json::from_str(BUILTIN).expect("valid growth path")
}

// ---- Records -------------------------------------------------------------

/// A card was read (first time, or again as a review).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthRead {
    pub path_version: String,
    pub step: String,
    pub card: String,
    pub review: bool,
}

/// An exercise done. Texts are device only, never exported for research.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthPractice {
    pub path_version: String,
    pub step: String,
    pub practice: String,
    /// Id of a `Choice` (circle, quadrant, dimension, …).
    pub choice: Option<String>,
    /// Id of a linked record (a task for the matrix). Device only.
    pub ref_id: Option<String>,
    /// Prompt id → answer. Free text: device only.
    pub texts: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    /// "La sto scoprendo"
    Discovering,
    /// "La pratico a volte"
    Sometimes,
    /// "Mi viene naturale"
    Natural,
}

impl Stage {
    pub const ALL: [Stage; 3] = [Self::Discovering, Self::Sometimes, Self::Natural];
}

/// How the habit feels now, in words; the latest one counts. Optional.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthStage {
    pub path_version: String,
    pub step: String,
    pub stage: Stage,
}

// ---- Progress ------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    NotStarted,
    Started,
    /// Every card read and at least one exercise done. Not "finished":
    /// practice and reviews go on.
    Explored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepProgress {
    pub step: String,
    pub cards_read: BTreeSet<String>,
    pub practices_done: BTreeSet<String>,
    pub stage: Option<Stage>,
    pub status: StepStatus,
}

fn newest_first(events: &[Event]) -> Vec<&Event> {
    let mut v: Vec<&Event> = events.iter().collect();
    v.sort_by_key(|e| std::cmp::Reverse(e.timestamp()));
    v
}

/// Progress of every step, in path order. Records of ids no longer in the
/// path are ignored.
pub fn progress(path: &Path, events: &[Event]) -> Vec<StepProgress> {
    let newest = newest_first(events);
    path.steps
        .iter()
        .map(|s| {
            let mut cards_read = BTreeSet::new();
            let mut practices_done = BTreeSet::new();
            let mut stage = None;
            for e in &newest {
                match &e.payload {
                    Payload::GrowthRead(r) if r.step == s.id && s.card(&r.card).is_some() => {
                        cards_read.insert(r.card.clone());
                    }
                    Payload::GrowthPractice(p)
                        if p.step == s.id && s.practice(&p.practice).is_some() =>
                    {
                        practices_done.insert(p.practice.clone());
                    }
                    Payload::GrowthStage(g) if g.step == s.id && stage.is_none() => {
                        stage = Some(g.stage);
                    }
                    _ => {}
                }
            }
            let status = if cards_read.len() == s.cards.len() && !practices_done.is_empty() {
                StepStatus::Explored
            } else if cards_read.is_empty() && practices_done.is_empty() {
                StepStatus::NotStarted
            } else {
                StepStatus::Started
            };
            StepProgress {
                step: s.id.clone(),
                cards_read,
                practices_done,
                stage,
                status,
            }
        })
        .collect()
}

/// The suggested step: the first not yet explored, in book order
/// (the last one when all are explored). A suggestion, never a lock.
pub fn suggested<'a>(path: &'a Path, events: &[Event]) -> &'a Step {
    let prog = progress(path, events);
    path.steps
        .iter()
        .zip(&prog)
        .find(|(_, p)| p.status != StepStatus::Explored)
        .map(|(s, _)| s)
        .unwrap_or_else(|| path.steps.last().expect("path has steps"))
}

/// The latest version of the personal mission, if any.
pub fn mission(events: &[Event]) -> Option<(&Event, &GrowthPractice)> {
    newest_first(events)
        .into_iter()
        .find_map(|e| match &e.payload {
            Payload::GrowthPractice(p) if p.practice == "missione" => Some((e, p)),
            _ => None,
        })
}

/// The roles from the latest mission, one per line.
pub fn roles(events: &[Event]) -> Vec<String> {
    mission(events)
        .and_then(|(_, p)| p.texts.get("ruoli"))
        .map(|r| {
            r.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

// ---- Time matrix (habit 3 ↔ tasks) ---------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Quadrant {
    Q1,
    Q2,
    Q3,
    Q4,
}

impl Quadrant {
    pub const ALL: [Quadrant; 4] = [Self::Q1, Self::Q2, Self::Q3, Self::Q4];
    pub fn id(self) -> &'static str {
        match self {
            Self::Q1 => "q1",
            Self::Q2 => "q2",
            Self::Q3 => "q3",
            Self::Q4 => "q4",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|q| q.id() == id)
    }
}

/// The latest quadrant of every task placed in the matrix (task id → quadrant).
/// A placement with no choice takes the task out of the matrix.
pub fn matrix(events: &[Event]) -> BTreeMap<String, Quadrant> {
    let mut out = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for e in newest_first(events) {
        if let Payload::GrowthPractice(p) = &e.payload
            && p.practice == "matrice"
            && let Some(task) = &p.ref_id
            && seen.insert(task.clone())
            && let Some(q) = p.choice.as_deref().and_then(Quadrant::from_id)
        {
            out.insert(task.clone(), q);
        }
    }
    out
}

/// Open tasks not yet placed in the matrix, oldest first.
pub fn unplaced_tasks(events: &[Event]) -> Vec<&Event> {
    let placed = matrix(events);
    let mut v: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(&e.payload, Payload::Task(t) if t.is_open()))
        .filter(|e| !placed.contains_key(&e.id))
        .collect();
    v.sort_by_key(|e| e.timestamp());
    v
}

// ---- Renewal (habit 7 ↔ care gestures) -----------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dimension {
    Physical,
    Mental,
    Social,
    Spiritual,
}

impl Dimension {
    pub const ALL: [Dimension; 4] = [Self::Physical, Self::Mental, Self::Social, Self::Spiritual];
    /// The choice id in the path file.
    pub fn id(self) -> &'static str {
        match self {
            Self::Physical => "fisica",
            Self::Mental => "mentale",
            Self::Social => "sociale",
            Self::Spiritual => "spirituale",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.id() == id)
    }
    fn index(self) -> usize {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(0)
    }
}

/// Renewal gestures per dimension between `from` and `to` (half-open):
/// renewal exercises plus care gestures whose label the user linked to a
/// dimension (`links`: care label → dimension, in the settings).
/// A picture of balance, not a score: the app shows it without targets.
pub fn renewal(
    events: &[Event],
    links: &BTreeMap<String, Dimension>,
    from: jiff::Timestamp,
    to: jiff::Timestamp,
) -> [usize; 4] {
    let mut out = [0; 4];
    for e in events {
        let t = e.timestamp();
        if t < from || t >= to {
            continue;
        }
        let dim = match &e.payload {
            Payload::GrowthPractice(p) if p.practice == "rinnovamento" => {
                p.choice.as_deref().and_then(Dimension::from_id)
            }
            Payload::Care(c) => links.get(&c.label).copied(),
            _ => None,
        };
        if let Some(d) = dim {
            out[d.index()] += 1;
        }
    }
    out
}

// ---- Reviews -------------------------------------------------------------

/// Days until the next review, by how many times the card has been read.
pub const REVIEW_DAYS: [i64; 5] = [3, 7, 21, 60, 120];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Due {
    pub step: String,
    pub card: String,
    /// How many times it was read so far.
    pub reads: usize,
}

/// Cards whose review time has come, most overdue first. Only cards read at
/// least once: the path itself is never pushed. The app offers one at a time,
/// with "non ora".
pub fn due_reviews(path: &Path, events: &[Event], now: jiff::Timestamp) -> Vec<Due> {
    review_times(path, events)
        .into_iter()
        .filter(|(at, _)| *at <= now)
        .map(|(_, d)| d)
        .collect()
}

/// When each card read at least once comes up for review (past or future),
/// soonest first.
pub fn review_times(path: &Path, events: &[Event]) -> Vec<(jiff::Timestamp, Due)> {
    let mut reads: BTreeMap<(String, String), (usize, jiff::Timestamp)> = BTreeMap::new();
    for e in events {
        if let Payload::GrowthRead(r) = &e.payload {
            let t = e.timestamp();
            let entry = reads
                .entry((r.step.clone(), r.card.clone()))
                .or_insert((0, t));
            entry.0 += 1;
            entry.1 = entry.1.max(t);
        }
    }
    let mut out: Vec<(jiff::Timestamp, Due)> = reads
        .into_iter()
        .filter(|((s, c), _)| path.step(s).and_then(|s| s.card(c)).is_some())
        .map(|((step, card), (n, last))| {
            let days = REVIEW_DAYS[(n - 1).min(REVIEW_DAYS.len() - 1)];
            let at = last + jiff::SignedDuration::from_hours(days * 24);
            (
                at,
                Due {
                    step,
                    card,
                    reads: n,
                },
            )
        })
        .collect();
    out.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| (&a.1.step, &a.1.card).cmp(&(&b.1.step, &b.1.card)))
    });
    out
}

// ---- The week --------------------------------------------------------------

/// What happened on the path in a span of time, for the weekly summary.
/// Counts only, never texts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrowthWeek {
    /// Cards read for the first time.
    pub cards_read: usize,
    /// Cards read again (reviews).
    pub cards_reviewed: usize,
    /// (step id, practice id, times), most frequent first.
    pub practices: Vec<(String, String, usize)>,
    /// Renewal per dimension, in `Dimension::ALL` order.
    pub renewal: [usize; 4],
}

impl GrowthWeek {
    pub fn is_empty(&self) -> bool {
        self.cards_read == 0
            && self.cards_reviewed == 0
            && self.practices.is_empty()
            && self.renewal.iter().all(|n| *n == 0)
    }
}

/// The path between `from` and `to` (half-open). A read counts as a review
/// when the same card had been read before it.
pub fn week(
    events: &[Event],
    links: &BTreeMap<String, Dimension>,
    from: jiff::Timestamp,
    to: jiff::Timestamp,
) -> GrowthWeek {
    let mut w = GrowthWeek {
        renewal: renewal(events, links, from, to),
        ..GrowthWeek::default()
    };
    let mut first_read: BTreeMap<(&str, &str), jiff::Timestamp> = BTreeMap::new();
    for e in events {
        if let Payload::GrowthRead(r) = &e.payload {
            let t = e.timestamp();
            let slot = first_read
                .entry((r.step.as_str(), r.card.as_str()))
                .or_insert(t);
            *slot = (*slot).min(t);
        }
    }
    let mut practices: BTreeMap<(String, String), usize> = BTreeMap::new();
    for e in events {
        let t = e.timestamp();
        if t < from || t >= to {
            continue;
        }
        match &e.payload {
            Payload::GrowthRead(r) => {
                if first_read.get(&(r.step.as_str(), r.card.as_str())) == Some(&t) {
                    w.cards_read += 1;
                } else {
                    w.cards_reviewed += 1;
                }
            }
            Payload::GrowthPractice(p) => {
                *practices
                    .entry((p.step.clone(), p.practice.clone()))
                    .or_default() += 1;
            }
            _ => {}
        }
    }
    let mut v: Vec<_> = practices.into_iter().map(|((s, p), n)| (s, p, n)).collect();
    v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| (&a.0, &a.1).cmp(&(&b.0, &b.1))));
    w.practices = v;
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::{Care, Task};

    /// Local time in Rome; the offset in the test strings is only for reading.
    fn zoned(iso: &str) -> jiff::ZonedDateTime {
        let dt: jiff::civil::DateTime = iso[..19].parse().unwrap();
        dt.to_zoned_datetime(jiff::tz::TimeZone::get("Europe/Rome").unwrap())
            .unwrap()
    }

    fn at(p: Payload, iso: &str) -> Event {
        Event::at(p, Origin::Spontaneous, zoned(iso))
    }

    fn read(step: &str, card: &str, iso: &str) -> Event {
        at(
            Payload::GrowthRead(GrowthRead {
                path_version: path().version,
                step: step.into(),
                card: card.into(),
                review: false,
            }),
            iso,
        )
    }

    fn practice(
        step: &str,
        id: &str,
        choice: Option<&str>,
        ref_id: Option<&str>,
    ) -> GrowthPractice {
        GrowthPractice {
            path_version: path().version,
            step: step.into(),
            practice: id.into(),
            choice: choice.map(String::from),
            ref_id: ref_id.map(String::from),
            texts: BTreeMap::new(),
        }
    }

    fn ts(iso: &str) -> jiff::Timestamp {
        zoned(iso).timestamp()
    }

    #[test]
    fn the_path_is_complete_and_consistent() {
        let p = path();
        assert!(p.version.starts_with("crescita-v"));
        assert_eq!(p.source.author, "Stephen R. Covey");
        let numbers: Vec<u8> = p.steps.iter().map(|s| s.number).collect();
        assert_eq!(numbers, (0..=7).collect::<Vec<_>>());
        let mut step_ids = BTreeSet::new();
        for s in &p.steps {
            assert!(step_ids.insert(&s.id), "duplicate step {}", s.id);
            assert!(s.cards.len() >= 3, "{}: too few cards", s.id);
            assert!(s.practice("spiega").is_some(), "{}: no 'spiega'", s.id);
            let cards: BTreeSet<_> = s.cards.iter().map(|c| &c.id).collect();
            assert_eq!(cards.len(), s.cards.len(), "{}: duplicate card", s.id);
            let practices: BTreeSet<_> = s.practices.iter().map(|c| &c.id).collect();
            assert_eq!(
                practices.len(),
                s.practices.len(),
                "{}: duplicate practice",
                s.id
            );
            for c in &s.cards {
                assert!(!c.title.is_empty() && !c.text.is_empty());
            }
            for pr in &s.practices {
                let prompts: BTreeSet<_> = pr.prompts.iter().map(|q| &q.id).collect();
                assert_eq!(prompts.len(), pr.prompts.len(), "{}.{}", s.id, pr.id);
                match pr.kind {
                    PracticeKind::Write | PracticeKind::Mission => {
                        assert!(!pr.prompts.is_empty() && pr.choices.is_empty())
                    }
                    PracticeKind::Choose => assert!(pr.choices.len() >= 2),
                    PracticeKind::Matrix => {
                        let ids: Vec<_> = pr.choices.iter().map(|c| c.id.as_str()).collect();
                        assert_eq!(ids, Quadrant::ALL.map(Quadrant::id));
                    }
                    PracticeKind::Renewal => {
                        let ids: Vec<_> = pr.choices.iter().map(|c| c.id.as_str()).collect();
                        assert_eq!(ids, Dimension::ALL.map(Dimension::id));
                    }
                }
            }
        }
        // The links in code point at real ids.
        assert_eq!(
            p.step("fine-in-mente")
                .unwrap()
                .practice("missione")
                .unwrap()
                .kind,
            PracticeKind::Mission
        );
        assert!(
            p.step("fine-in-mente")
                .unwrap()
                .practice("missione")
                .unwrap()
                .prompts
                .iter()
                .any(|q| q.id == "ruoli")
        );
        assert_eq!(
            p.step("prima-importanti")
                .unwrap()
                .practice("matrice")
                .unwrap()
                .kind,
            PracticeKind::Matrix
        );
        assert_eq!(
            p.step("affilare")
                .unwrap()
                .practice("rinnovamento")
                .unwrap()
                .kind,
            PracticeKind::Renewal
        );
        assert!(!BUILTIN.contains("SEGNAPOSTO"));
    }

    #[test]
    fn progress_and_the_suggested_step() {
        let p = path();
        assert_eq!(suggested(&p, &[]).id, "basi");
        let basi = p.step("basi").unwrap();
        let mut events: Vec<Event> = basi
            .cards
            .iter()
            .map(|c| read("basi", &c.id, "2026-11-01T10:00:00+01:00"))
            .collect();
        let prog = progress(&p, &events);
        assert_eq!(prog[0].status, StepStatus::Started); // read, nothing practised
        assert_eq!(suggested(&p, &events).id, "basi");
        events.push(at(
            Payload::GrowthPractice(practice("basi", "spiega", None, None)),
            "2026-11-01T11:00:00+01:00",
        ));
        events.push(at(
            Payload::GrowthStage(GrowthStage {
                path_version: p.version.clone(),
                step: "basi".into(),
                stage: Stage::Discovering,
            }),
            "2026-11-01T11:00:00+01:00",
        ));
        events.push(at(
            Payload::GrowthStage(GrowthStage {
                path_version: p.version.clone(),
                step: "basi".into(),
                stage: Stage::Sometimes,
            }),
            "2026-11-03T11:00:00+01:00",
        ));
        // Unknown ids are ignored, and do not start a step.
        events.push(read("proattivi", "non-esiste", "2026-11-02T10:00:00+01:00"));
        let prog = progress(&p, &events);
        assert_eq!(prog[0].status, StepStatus::Explored);
        assert_eq!(prog[0].stage, Some(Stage::Sometimes));
        assert_eq!(prog[1].status, StepStatus::NotStarted);
        // Not locked: practising a later habit is allowed, the suggestion follows the book.
        events.push(at(
            Payload::GrowthPractice(practice("affilare", "rinnovamento", Some("fisica"), None)),
            "2026-11-02T10:00:00+01:00",
        ));
        assert_eq!(progress(&p, &events)[7].status, StepStatus::Started);
        assert_eq!(suggested(&p, &events).id, "proattivi");
    }

    #[test]
    fn the_latest_mission_wins_and_keeps_the_old_ones() {
        let mut old = practice("fine-in-mente", "missione", None, None);
        old.texts.insert("chi".into(), "prima".into());
        let mut new = practice("fine-in-mente", "missione", None, None);
        new.texts.insert("chi".into(), "dopo".into());
        new.texts
            .insert("ruoli".into(), "Figlio\n\n  Allevatore \nAmico".into());
        let events = vec![
            at(Payload::GrowthPractice(new), "2026-11-10T09:00:00+01:00"),
            at(Payload::GrowthPractice(old), "2026-11-01T09:00:00+01:00"),
        ];
        assert_eq!(mission(&events).unwrap().1.texts["chi"], "dopo");
        assert_eq!(roles(&events), ["Figlio", "Allevatore", "Amico"]);
        assert!(mission(&[]).is_none());
        assert!(roles(&[]).is_empty());
    }

    #[test]
    fn matrix_keeps_the_latest_placement_per_task() {
        let t1 = at(
            Payload::Task(Task::new("Potare")),
            "2026-11-01T09:00:00+01:00",
        );
        let t2 = at(
            Payload::Task(Task::new("Bollette")),
            "2026-11-02T09:00:00+01:00",
        );
        let mut done = Task::new("Fatto");
        done.status = crate::kinds::TaskStatus::Done;
        let t3 = at(Payload::Task(done), "2026-11-02T09:00:00+01:00");
        let place = |task: &Event, q: Option<&str>, iso| {
            at(
                Payload::GrowthPractice(practice("prima-importanti", "matrice", q, Some(&task.id))),
                iso,
            )
        };
        let mut events = vec![
            place(&t1, Some("q3"), "2026-11-03T09:00:00+01:00"),
            place(&t1, Some("q2"), "2026-11-04T09:00:00+01:00"),
        ];
        let m = matrix(&events);
        assert_eq!(m[&t1.id], Quadrant::Q2);
        events.extend([t1.clone(), t2.clone(), t3]);
        let unplaced: Vec<_> = unplaced_tasks(&events)
            .iter()
            .map(|e| e.id.clone())
            .collect();
        assert_eq!(unplaced, [t2.id]); // done tasks are not offered
        // Taking a task out of the matrix.
        events.push(place(&t1, None, "2026-11-05T09:00:00+01:00"));
        assert!(matrix(&events).is_empty());
    }

    #[test]
    fn renewal_counts_exercises_and_linked_care_gestures() {
        let care = |l: &str, iso| {
            at(
                Payload::Care(Care {
                    label: l.into(),
                    duration_min: None,
                    fulfilment: None,
                }),
                iso,
            )
        };
        let events = vec![
            care("Camminata", "2026-11-16T09:00:00+01:00"),
            care("Camminata", "2026-11-18T09:00:00+01:00"),
            care("Lettura", "2026-11-17T09:00:00+01:00"),
            care("Senza dimensione", "2026-11-17T09:00:00+01:00"),
            care("Camminata", "2026-11-23T09:00:00+01:00"), // next week
            at(
                Payload::GrowthPractice(practice(
                    "affilare",
                    "rinnovamento",
                    Some("spirituale"),
                    None,
                )),
                "2026-11-19T09:00:00+01:00",
            ),
        ];
        let links = BTreeMap::from([
            ("Camminata".to_string(), Dimension::Physical),
            ("Lettura".to_string(), Dimension::Mental),
        ]);
        let week = renewal(
            &events,
            &links,
            ts("2026-11-16T00:00:00+01:00"),
            ts("2026-11-23T00:00:00+01:00"),
        );
        assert_eq!(week, [2, 1, 0, 1]);
    }

    #[test]
    fn reviews_come_back_on_expanding_intervals() {
        let p = path();
        let first = read("proattivi", "cerchi", "2026-11-01T10:00:00+01:00");
        assert!(
            due_reviews(
                &p,
                std::slice::from_ref(&first),
                ts("2026-11-03T10:00:00+01:00")
            )
            .is_empty()
        );
        let due = due_reviews(
            &p,
            std::slice::from_ref(&first),
            ts("2026-11-04T10:00:00+01:00"),
        );
        assert_eq!(
            due,
            [Due {
                step: "proattivi".into(),
                card: "cerchi".into(),
                reads: 1
            }]
        );
        // Reviewed on the 4th: next one 7 days later.
        let mut again = read("proattivi", "cerchi", "2026-11-04T10:00:00+01:00");
        if let Payload::GrowthRead(r) = &mut again.payload {
            r.review = true;
        }
        let events = vec![first, again];
        assert!(due_reviews(&p, &events, ts("2026-11-10T10:00:00+01:00")).is_empty());
        assert_eq!(
            due_reviews(&p, &events, ts("2026-11-11T10:00:00+01:00")).len(),
            1
        );
        // Most overdue first; cards no longer in the path never come back.
        let older = read("basi", "paradigmi", "2026-10-01T10:00:00+01:00");
        let gone = read("basi", "tolta", "2026-10-01T10:00:00+01:00");
        let all = [events, vec![older, gone]].concat();
        let due = due_reviews(&p, &all, ts("2026-11-20T10:00:00+01:00"));
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].card, "paradigmi");
        assert_eq!(REVIEW_DAYS.last(), Some(&120)); // never more often than this, forever
    }

    #[test]
    fn the_week_counts_reads_reviews_practices_and_renewal() {
        let links = BTreeMap::from([("Camminare".to_string(), Dimension::Physical)]);
        let care = |label: &str, iso: &str| {
            at(
                Payload::Care(Care {
                    label: label.into(),
                    duration_min: None,
                    fulfilment: None,
                }),
                iso,
            )
        };
        let events = vec![
            read("proattivi", "cerchi", "2026-11-10T10:00:00+01:00"), // week before
            read("proattivi", "cerchi", "2026-11-17T10:00:00+01:00"), // review
            read("proattivi", "linguaggio", "2026-11-18T10:00:00+01:00"),
            at(
                Payload::GrowthPractice(practice(
                    "affilare",
                    "rinnovamento",
                    Some("sociale"),
                    None,
                )),
                "2026-11-19T10:00:00+01:00",
            ),
            at(
                Payload::GrowthPractice(practice("proattivi", "cerchio", None, None)),
                "2026-11-20T10:00:00+01:00",
            ),
            care("Camminare", "2026-11-21T10:00:00+01:00"),
            care("Leggere", "2026-11-21T11:00:00+01:00"), // not linked
        ];
        let w = week(
            &events,
            &links,
            ts("2026-11-16T00:00:00+01:00"),
            ts("2026-11-23T00:00:00+01:00"),
        );
        assert_eq!((w.cards_read, w.cards_reviewed), (1, 1));
        assert_eq!(w.practices.len(), 2);
        assert_eq!(w.renewal, [1, 0, 1, 0]);
        assert!(!w.is_empty());
        let empty = week(
            &events,
            &links,
            ts("2026-12-01T00:00:00+01:00"),
            ts("2026-12-08T00:00:00+01:00"),
        );
        assert!(empty.is_empty());
    }
}
