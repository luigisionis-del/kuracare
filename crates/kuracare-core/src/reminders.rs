//! Invitations ("Ti va di…?"): when the phone offers a check-in, or recalls
//! something to do at its time.
//!
//! The plan holds only instants, a key and generic texts. Nothing personal goes
//! into it (never a task title), because the Android side keeps it unencrypted:
//! it must work while the app is locked and after a reboot.
//!
//! - One invitation per check-in window and day, at a random moment of the
//!   window (signal-contingent sampling, as in EMA). The moment is derived from a
//!   per-install salt, so recomputing the plan gives the same moments.
//! - No invitation for a window that already has a check-in, during a pause, or
//!   once over. Never repeated: it disappears at the end of its window. An
//!   invitation still showing stays in the plan until then (the Android side
//!   removes notifications that drop out of it, e.g. after a check-in).
//! - Growth reviews (opt-in): one invitation on the day a card of the path
//!   comes up for review, at the chosen time. A card already overdue never
//!   invites again (Home and Crescita still offer it, with "Non ora").
//! - The plan covers `PLAN_DAYS` days; if the app is not opened for that long,
//!   invitations simply stop.

use jiff::civil::Date;
use jiff::{SignedDuration, Timestamp, ZonedDateTime};
use serde::Serialize;

use crate::event::Event;
use crate::kinds::Payload;
use crate::settings::{Settings, TimeWindow};
use crate::time;

pub const PLAN_VERSION: u32 = 1;
pub const PLAN_DAYS: i64 = 14;
/// An invitation never comes in the last minutes of its window.
const END_MARGIN: SignedDuration = SignedDuration::from_mins(15);
/// How long a task invitation stays in the notification shade.
const TASK_SHOWN: SignedDuration = SignedDuration::from_hours(3);

pub const CHECKIN_TITLE: &str = "Ti va di fermarti un momento?";
pub const CHECKIN_BODY: &str = "Un check-in breve, se ti va. Ogni voce si può saltare.";
pub const TASK_TITLE: &str = "C'è una cosa in programma per adesso";
pub const TASK_BODY: &str = "Ti va di darle un'occhiata?";
pub const REVIEW_TITLE: &str = "Ti va di ripassare un'idea?";
pub const REVIEW_BODY: &str = "Una scheda della Crescita, un paio di minuti.";
/// How long a review invitation stays in the notification shade.
const REVIEW_SHOWN: SignedDuration = SignedDuration::from_hours(3);
pub const NOT_NOW: &str = "Non ora";
pub const CHANNEL: &str = "Inviti";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InviteKind {
    Checkin,
    Task,
    Review,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    /// Stable identity, e.g. `checkin:2026-11-21:0` or `task:evt:<uuid>`.
    pub key: String,
    pub kind: InviteKind,
    /// Unix milliseconds.
    pub at: i64,
    /// Unix milliseconds: the notification goes away by itself then.
    pub until: i64,
    pub title: &'static str,
    pub body: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub version: u32,
    pub channel: &'static str,
    pub not_now: &'static str,
    pub items: Vec<Invite>,
}

impl Plan {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("plan serialises")
    }
}

/// splitmix64: a well-known, stable mixing function (deterministic offsets).
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hm(s: &str) -> Option<(i8, i8)> {
    Some((s.get(..2)?.parse().ok()?, s.get(3..5)?.parse().ok()?))
}

fn local(day: Date, hhmm: &str, now: &ZonedDateTime) -> Option<Timestamp> {
    let (h, m) = hm(hhmm)?;
    if !(0..24).contains(&h) || !(0..60).contains(&m) {
        return None;
    }
    day.at(h, m, 0, 0)
        .to_zoned_datetime(now.time_zone().clone())
        .ok()
        .map(|z| z.timestamp())
}

/// Start, end and the invitation moment of window `index` on `day`.
fn window_moment(
    salt: u64,
    day: Date,
    index: usize,
    w: &TimeWindow,
    now: &ZonedDateTime,
) -> Option<(Timestamp, Timestamp, Timestamp)> {
    if !w.is_valid() {
        return None;
    }
    let start = local(day, &w.start, now)?;
    let end = local(day, &w.end, now)?;
    let span = (end.duration_since(start) - END_MARGIN).as_secs().max(0) as u64;
    let day_num = (day.year() as u64) * 10_000 + (day.month() as u64) * 100 + day.day() as u64;
    let r = mix(salt ^ mix(day_num.wrapping_mul(31).wrapping_add(index as u64)));
    let at = start + SignedDuration::from_secs((r % (span + 1)) as i64);
    Some((start, end, at))
}

fn paused_until(s: &Settings) -> Option<Timestamp> {
    s.pause_until.as_deref().and_then(|p| time::parse(p).ok())
}

/// The plan for the next `PLAN_DAYS` days, sorted by time. Empty when
/// invitations are off.
pub fn plan(settings: &Settings, events: &[Event], now: &ZonedDateTime) -> Plan {
    let mut items = vec![];
    if settings.invitations {
        let now_ts = now.timestamp();
        let pause = paused_until(settings);
        let paused = |t: Timestamp| pause.is_some_and(|p| t < p);
        let checkins: Vec<Timestamp> = events
            .iter()
            .filter(|e| matches!(e.payload, Payload::Checkin(_)))
            .map(Event::timestamp)
            .collect();

        let mut day = now.date();
        for _ in 0..PLAN_DAYS {
            for (i, w) in settings.checkin_windows.iter().enumerate() {
                let Some((start, end, at)) = window_moment(settings.invite_salt, day, i, w, now)
                else {
                    continue;
                };
                let answered = checkins.iter().any(|t| (start..=end).contains(t));
                if end <= now_ts || paused(at) || answered {
                    continue;
                }
                items.push(Invite {
                    key: format!("checkin:{day}:{i}"),
                    kind: InviteKind::Checkin,
                    at: at.as_millisecond(),
                    until: end.as_millisecond(),
                    title: CHECKIN_TITLE,
                    body: CHECKIN_BODY,
                });
            }
            let Ok(next) = day.tomorrow() else { break };
            day = next;
        }

        let horizon = now_ts + SignedDuration::from_hours(24 * PLAN_DAYS);
        for e in events {
            let Payload::Task(t) = &e.payload else {
                continue;
            };
            let (Some(date), Some(hhmm)) = (&t.due_date, &t.due_time) else {
                continue;
            };
            let Some(at) = date.parse::<Date>().ok().and_then(|d| local(d, hhmm, now)) else {
                continue;
            };
            let until = at + TASK_SHOWN;
            if !t.is_open() || until <= now_ts || at > horizon || paused(at) {
                continue;
            }
            items.push(Invite {
                key: format!("task:{}", e.id),
                kind: InviteKind::Task,
                at: at.as_millisecond(),
                until: until.as_millisecond(),
                title: TASK_TITLE,
                body: TASK_BODY,
            });
        }
        if settings.growth_invites {
            items.extend(review_invites(settings, events, now, &paused));
        }
        items.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.key.cmp(&b.key)));
    }
    Plan {
        version: PLAN_VERSION,
        channel: CHANNEL,
        not_now: NOT_NOW,
        items,
    }
}

/// One invitation per day on which a card comes up for review (in the 24
/// hours before that day's invitation time). Today's is skipped once a card
/// was read today or the review got a "Non ora".
fn review_invites(
    settings: &Settings,
    events: &[Event],
    now: &ZonedDateTime,
    paused: &dyn Fn(Timestamp) -> bool,
) -> Vec<Invite> {
    let times: Vec<Timestamp> = crate::growth::review_times(&crate::growth::path(), events)
        .into_iter()
        .map(|(t, _)| t)
        .collect();
    if times.is_empty() {
        return vec![];
    }
    let today = now.date();
    let today_s = today.to_string();
    let quiet_today = settings.growth_review_dismissed_on.as_deref() == Some(today_s.as_str())
        || events.iter().any(|e| {
            matches!(e.payload, Payload::GrowthRead(_))
                && e.timestamp()
                    .to_zoned_datetime(now.time_zone().clone())
                    .date()
                    == today
        });
    let mut out = vec![];
    let mut day = today;
    for _ in 0..PLAN_DAYS {
        if let Some(at) = local(day, &settings.growth_invite_time, now) {
            let since = at - SignedDuration::from_hours(24);
            let until = at + REVIEW_SHOWN;
            let comes_up = times.iter().any(|t| since < *t && *t <= at);
            if comes_up && until > now.timestamp() && !paused(at) && !(day == today && quiet_today)
            {
                out.push(Invite {
                    key: format!("review:{day}"),
                    kind: InviteKind::Review,
                    at: at.as_millisecond(),
                    until: until.as_millisecond(),
                    title: REVIEW_TITLE,
                    body: REVIEW_BODY,
                });
            }
        }
        let Ok(next) = day.tomorrow() else { break };
        day = next;
    }
    out
}

/// A fresh random salt for a new install (never 0, which means "not set").
pub fn new_salt() -> u64 {
    loop {
        let s: u64 = rand::random();
        if s != 0 {
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::{Checkin, Task, TaskStatus};
    use jiff::tz::TimeZone;

    fn rome(s: &str) -> ZonedDateTime {
        let dt: jiff::civil::DateTime = s.parse().unwrap();
        dt.to_zoned_datetime(TimeZone::get("Europe/Rome").unwrap())
            .unwrap()
    }

    fn settings() -> Settings {
        Settings {
            invitations: true,
            invite_salt: 42,
            checkin_windows: vec![
                TimeWindow {
                    start: "09:00".into(),
                    end: "11:00".into(),
                },
                TimeWindow {
                    start: "18:00".into(),
                    end: "20:00".into(),
                },
            ],
            ..Settings::default()
        }
    }

    fn ts(ms: i64) -> ZonedDateTime {
        Timestamp::from_millisecond(ms)
            .unwrap()
            .to_zoned_datetime(TimeZone::get("Europe/Rome").unwrap())
    }

    #[test]
    fn off_means_nothing() {
        let s = Settings {
            invitations: false,
            ..settings()
        };
        assert!(plan(&s, &[], &rome("2026-11-21T07:00")).items.is_empty());
    }

    #[test]
    fn one_invitation_per_window_inside_it_and_stable() {
        let now = rome("2026-11-21T07:00");
        let p = plan(&settings(), &[], &now);
        assert_eq!(p.items.len(), 2 * PLAN_DAYS as usize);
        for it in &p.items {
            let z = ts(it.at);
            let hm = z.strftime("%H:%M").to_string();
            let until = ts(it.until).strftime("%H:%M").to_string();
            assert!(
                ("09:00".."10:46").contains(&hm.as_str()) && until == "11:00"
                    || ("18:00".."19:46").contains(&hm.as_str()) && until == "20:00",
                "{} {hm} {until}",
                it.key
            );
        }
        assert_eq!(p.items[0].key, "checkin:2026-11-21:0");
        // Same salt, same moments; another salt moves them.
        assert_eq!(plan(&settings(), &[], &now), p);
        let other = Settings {
            invite_salt: 7,
            ..settings()
        };
        assert_ne!(plan(&other, &[], &now).items[0].at, p.items[0].at);
        // Moments are not all the same time of day.
        let mins: std::collections::HashSet<_> = p
            .items
            .iter()
            .map(|i| ts(i.at).strftime("%H:%M").to_string())
            .collect();
        assert!(mins.len() > PLAN_DAYS as usize);
    }

    #[test]
    fn skips_past_answered_and_paused() {
        let now = rome("2026-11-21T12:00");
        let checkin = Event::at(
            Payload::Checkin(Checkin::empty()),
            Origin::Spontaneous,
            rome("2026-11-21T18:30"),
        );
        let p = plan(&settings(), &[checkin], &now);
        assert!(
            p.items
                .iter()
                .all(|i| !i.key.starts_with("checkin:2026-11-21"))
        );
        assert_eq!(p.items[0].key, "checkin:2026-11-22:0");

        // Inside the evening window, unanswered: still in the plan until 20:00.
        let p = plan(&settings(), &[], &rome("2026-11-21T19:59"));
        assert_eq!(p.items[0].key, "checkin:2026-11-21:1");

        let mut s = settings();
        s.pause_until = Some(time::format(&rome("2026-11-24T00:00")));
        let p = plan(&s, &[], &now);
        assert_eq!(p.items[0].key, "checkin:2026-11-24:0");
    }

    #[test]
    fn tasks_with_a_time_never_carry_their_title() {
        let now = rome("2026-11-21T07:00");
        let task = |time: Option<&str>, status| {
            Event::at(
                Payload::Task(Task {
                    due_date: Some("2026-11-21".into()),
                    due_time: time.map(Into::into),
                    status,
                    ..Task::new("Chiamare la farmacia")
                }),
                Origin::Spontaneous,
                now.clone(),
            )
        };
        let events = [
            task(Some("15:30"), TaskStatus::Open),
            task(None, TaskStatus::Open),
            task(Some("16:00"), TaskStatus::Done),
            task(Some("03:00"), TaskStatus::Open),
        ];
        let p = plan(&settings(), &events, &now);
        let tasks: Vec<_> = p
            .items
            .iter()
            .filter(|i| i.kind == InviteKind::Task)
            .collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(ts(tasks[0].at).strftime("%H:%M").to_string(), "15:30");
        assert_eq!(tasks[0].key, format!("task:{}", events[0].id));
        let json = p.to_json();
        assert!(!json.contains("farmacia"));
        assert!(json.contains("\"notNow\":\"Non ora\""));
    }

    #[test]
    fn survives_daylight_saving_changes() {
        // 29/03/2026: Rome jumps from 02:00 to 03:00.
        let mut s = settings();
        s.checkin_windows = vec![TimeWindow {
            start: "01:30".into(),
            end: "03:30".into(),
        }];
        let p = plan(&s, &[], &rome("2026-03-28T12:00"));
        assert!(p.items.iter().any(|i| i.key == "checkin:2026-03-29:0"));
    }

    #[test]
    fn review_invitations_come_once_on_the_day_a_card_is_due() {
        use crate::growth::{GrowthRead, path};
        let read = Event::at(
            Payload::GrowthRead(GrowthRead {
                path_version: path().version,
                step: "proattivi".into(),
                card: "cerchi".into(),
                review: false,
            }),
            Origin::Spontaneous,
            rome("2026-11-21T10:00"),
        );
        let mut s = settings();
        s.growth_invites = true;
        // Read on the 21st at 10:00: due on the 24th at 10:00, invited at 18:00.
        let reviews = |s: &Settings, now: &str| -> Vec<String> {
            plan(s, std::slice::from_ref(&read), &rome(now))
                .items
                .into_iter()
                .filter(|i| i.kind == InviteKind::Review)
                .map(|i| i.key)
                .collect()
        };
        assert_eq!(reviews(&s, "2026-11-21T12:00"), ["review:2026-11-24"]);
        // Overdue cards never invite again.
        assert!(reviews(&s, "2026-11-25T08:00").is_empty());
        // "Non ora" on the day itself.
        s.growth_review_dismissed_on = Some("2026-11-24".into());
        assert!(reviews(&s, "2026-11-24T09:00").is_empty());
        // Off by default.
        assert!(reviews(&settings(), "2026-11-21T12:00").is_empty());
        let json = plan(
            &{
                let mut s = settings();
                s.growth_invites = true;
                s
            },
            &[read],
            &rome("2026-11-21T12:00"),
        )
        .to_json();
        assert!(json.contains("\"kind\":\"review\""));
        assert!(!json.contains("cerchi"));
    }
}
