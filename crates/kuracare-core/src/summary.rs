//! Weekly summary (Monday to Sunday, local days).
//!
//! It opens with what did good (reflections, care gestures, the home, things
//! done), then the check-in trends, then simple associations. Nothing here is
//! advice or interpretation: means and counts only, and missing days stay
//! missing (never estimated, never pointed out).

use std::collections::BTreeMap;

use jiff::civil::Date;
use jiff::tz::TimeZone;

use crate::event::Event;
use crate::kinds::{CheckinItem, Payload, TaskStatus};

/// Associations look at the last `ASSOC_WEEKS` weeks, so that there are days to compare.
pub const ASSOC_WEEKS: i64 = 4;
/// Display rule (not a statistical threshold): a comparison is shown only when
/// both groups have at least this many days with a check-in.
pub const ASSOC_MIN_DAYS: usize = 3;

/// The Monday of the week that contains `d`.
pub fn week_start(d: Date) -> Date {
    let back = d.weekday().to_monday_zero_offset();
    d.checked_sub(jiff::Span::new().days(back)).unwrap_or(d)
}

fn day_of(e: &Event, tz: &TimeZone) -> Option<Date> {
    crate::time::parse(&e.ts)
        .ok()
        .map(|t| t.to_zoned_datetime(tz.clone()).date())
}

fn add_days(d: Date, n: i64) -> Date {
    d.checked_add(jiff::Span::new().days(n)).unwrap_or(d)
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Counted {
    pub label: String,
    pub times: usize,
    /// Mean of the fulfilment answers given (0-10), if any.
    pub fulfilment: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trend {
    pub item: CheckinItem,
    /// Mean per day, Monday first; None where there is no answer.
    pub days: [Option<f64>; 7],
    pub mean: Option<f64>,
    pub previous_mean: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FactorKind {
    Strain,
    Route,
    Care,
    HomeCare,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Association {
    pub kind: FactorKind,
    /// Strain name, route key ("oral"…), care label, or "" for home care.
    pub label: String,
    pub days_with: usize,
    pub days_without: usize,
    /// Means of the daily check-in values, in `CheckinItem::ALL` order.
    pub with: [Option<f64>; 5],
    pub without: [Option<f64>; 5],
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeekSummary {
    pub start: Date,
    pub end: Date,
    pub reflections: Vec<(Date, String)>,
    /// Care gestures, most frequent first.
    pub care: Vec<Counted>,
    pub home: Vec<Counted>,
    pub tasks_done: Vec<String>,
    pub checkin_days: usize,
    pub trends: Vec<Trend>,
    pub associations: Vec<Association>,
}

impl WeekSummary {
    pub fn has_good_things(&self) -> bool {
        !(self.reflections.is_empty()
            && self.care.is_empty()
            && self.home.is_empty()
            && self.tasks_done.is_empty())
    }
}

/// Per local day: the mean of each check-in item answered that day.
fn daily_checkins(events: &[Event], tz: &TimeZone) -> BTreeMap<Date, [Option<f64>; 5]> {
    let mut acc: BTreeMap<Date, [Vec<f64>; 5]> = BTreeMap::new();
    for e in events {
        let (Payload::Checkin(c), Some(d)) = (&e.payload, day_of(e, tz)) else {
            continue;
        };
        let slot = acc.entry(d).or_default();
        for (i, item) in CheckinItem::ALL.iter().enumerate() {
            if let Some(v) = item.get(c) {
                slot[i].push(f64::from(v));
            }
        }
    }
    acc.into_iter()
        .map(|(d, v)| (d, std::array::from_fn(|i| mean(&v[i]))))
        .filter(|(_, v): &(Date, [Option<f64>; 5])| v.iter().any(Option::is_some))
        .collect()
}

fn counted(map: BTreeMap<String, (usize, Vec<f64>)>) -> Vec<Counted> {
    let mut v: Vec<Counted> = map
        .into_iter()
        .map(|(label, (times, f))| Counted {
            label,
            times,
            fulfilment: mean(&f),
        })
        .collect();
    v.sort_by(|a, b| b.times.cmp(&a.times).then_with(|| a.label.cmp(&b.label)));
    v
}

fn route_key(r: crate::kinds::Route) -> &'static str {
    use crate::kinds::Route::*;
    match r {
        Inhalation => "inhalation",
        Vaporization => "vaporization",
        Oral => "oral",
        Sublingual => "sublingual",
    }
}

fn associations(
    events: &[Event],
    daily: &BTreeMap<Date, [Option<f64>; 5]>,
    from: Date,
    to: Date,
    tz: &TimeZone,
) -> Vec<Association> {
    // Which factors were present on which day.
    let mut present: BTreeMap<(u8, String), Vec<Date>> = BTreeMap::new();
    for e in events {
        let Some(d) = day_of(e, tz).filter(|d| (from..=to).contains(d)) else {
            continue;
        };
        let mut add = |k: u8, label: String| present.entry((k, label)).or_default().push(d);
        match &e.payload {
            Payload::Cannabis(c) => {
                add(0, c.strain.clone());
                if let Some(r) = c.route {
                    add(1, route_key(r).into());
                }
            }
            Payload::Care(c) => add(2, c.label.clone()),
            Payload::Home(_) => add(3, String::new()),
            _ => {}
        }
    }
    let days: Vec<(&Date, &[Option<f64>; 5])> = daily.range(from..=to).collect();
    let mut out = vec![];
    for ((k, label), mut on) in present {
        on.sort();
        on.dedup();
        let (with, without): (Vec<_>, Vec<_>) =
            days.iter().partition(|(d, _)| on.binary_search(d).is_ok());
        if with.len() < ASSOC_MIN_DAYS || without.len() < ASSOC_MIN_DAYS {
            continue;
        }
        let means = |group: &[&(&Date, &[Option<f64>; 5])]| -> [Option<f64>; 5] {
            std::array::from_fn(|i| {
                let v: Vec<f64> = group.iter().filter_map(|(_, vals)| vals[i]).collect();
                mean(&v)
            })
        };
        let with_refs: Vec<_> = with.iter().collect();
        let without_refs: Vec<_> = without.iter().collect();
        out.push(Association {
            kind: match k {
                0 => FactorKind::Strain,
                1 => FactorKind::Route,
                2 => FactorKind::Care,
                _ => FactorKind::HomeCare,
            },
            label,
            days_with: with.len(),
            days_without: without.len(),
            with: means(&with_refs),
            without: means(&without_refs),
        });
    }
    out
}

/// The summary of the week starting on `start` (a Monday), in zone `tz`.
pub fn weekly(events: &[Event], start: Date, tz: &TimeZone) -> WeekSummary {
    let start = week_start(start);
    let end = add_days(start, 6);
    let in_week = |e: &Event| day_of(e, tz).is_some_and(|d| (start..=end).contains(&d));

    let mut reflections = vec![];
    let mut care: BTreeMap<String, (usize, Vec<f64>)> = BTreeMap::new();
    let mut home: BTreeMap<String, (usize, Vec<f64>)> = BTreeMap::new();
    let mut tasks_done = vec![];
    for e in events.iter().filter(|e| in_week(e)) {
        match &e.payload {
            Payload::Reflection(r) => {
                reflections.push((day_of(e, tz).unwrap_or(start), r.text.clone()))
            }
            Payload::Care(c) => {
                let slot = care.entry(c.label.clone()).or_default();
                slot.0 += 1;
                slot.1.extend(c.fulfilment.map(f64::from));
            }
            Payload::Home(h) => {
                let slot = home.entry(h.room.clone()).or_default();
                slot.0 += 1;
                slot.1.extend(h.fulfilment.map(f64::from));
            }
            _ => {}
        }
    }
    // Things done count in the week they were closed, not when written down.
    for e in events {
        if let Payload::Task(t) = &e.payload
            && t.status == TaskStatus::Done
            && let Some(closed) = t
                .closed_at
                .as_deref()
                .and_then(|c| crate::time::parse(c).ok())
            && (start..=end).contains(&closed.to_zoned_datetime(tz.clone()).date())
        {
            tasks_done.push(t.title.clone());
        }
    }
    reflections.sort_by_key(|(d, _)| *d);

    let daily = daily_checkins(events, tz);
    let prev_start = add_days(start, -7);
    let trends = CheckinItem::ALL
        .iter()
        .enumerate()
        .map(|(i, &item)| {
            let days: [Option<f64>; 7] =
                std::array::from_fn(|k| daily.get(&add_days(start, k as i64)).and_then(|v| v[i]));
            let week_vals: Vec<f64> = days.iter().flatten().copied().collect();
            let prev_vals: Vec<f64> = daily
                .range(prev_start..start)
                .filter_map(|(_, v)| v[i])
                .collect();
            Trend {
                item,
                days,
                mean: mean(&week_vals),
                previous_mean: mean(&prev_vals),
            }
        })
        .collect();
    let checkin_days = daily.range(start..=end).count();
    let assoc_from = add_days(end, -(7 * ASSOC_WEEKS - 1));

    WeekSummary {
        start,
        end,
        reflections,
        care: counted(care),
        home: counted(home),
        tasks_done,
        checkin_days,
        trends,
        associations: associations(events, &daily, assoc_from, end, tz),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::*;

    fn tz() -> TimeZone {
        TimeZone::get("Europe/Rome").unwrap()
    }

    fn at(p: Payload, day: &str, hm: &str) -> Event {
        let dt: jiff::civil::DateTime = format!("{day}T{hm}").parse().unwrap();
        Event::at(p, Origin::Spontaneous, dt.to_zoned_datetime(tz()).unwrap())
    }

    fn checkin(mood: u8, load: Option<u8>) -> Payload {
        Payload::Checkin(Checkin {
            mood: Some(mood),
            mental_load: load,
            ..Checkin::empty()
        })
    }

    fn care(label: &str, f: Option<u8>) -> Payload {
        Payload::Care(Care {
            label: label.into(),
            duration_min: None,
            fulfilment: f,
        })
    }

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn weeks_start_on_monday() {
        assert_eq!(week_start(d("2026-11-21")), d("2026-11-16")); // Saturday
        assert_eq!(week_start(d("2026-11-16")), d("2026-11-16"));
        assert_eq!(week_start(d("2026-11-22")), d("2026-11-16")); // Sunday
    }

    #[test]
    fn opens_with_what_did_good() {
        let mut task = Task::new("Annaffiare");
        task.status = TaskStatus::Done;
        task.closed_at = Some("2026-11-18T10:00:00.000+01:00".into());
        let events = vec![
            at(care("Natura", Some(8)), "2026-11-16", "10:00"),
            at(care("Natura", Some(6)), "2026-11-18", "10:00"),
            at(care("Natura", None), "2026-11-19", "10:00"),
            at(care("Sonno", None), "2026-11-17", "10:00"),
            at(care("Sonno", None), "2026-11-10", "10:00"), // previous week
            at(
                Payload::Reflection(Reflection {
                    text: "il mare".into(),
                }),
                "2026-11-20",
                "21:00",
            ),
            // Written down the week before, done this week.
            at(Payload::Task(task), "2026-11-12", "09:00"),
        ];
        let s = weekly(&events, d("2026-11-19"), &tz());
        assert_eq!(s.start, d("2026-11-16"));
        assert_eq!(s.end, d("2026-11-22"));
        assert_eq!(s.care[0].label, "Natura");
        assert_eq!(s.care[0].times, 3);
        assert_eq!(s.care[0].fulfilment, Some(7.0));
        assert_eq!(s.care[1].times, 1);
        assert_eq!(
            s.reflections,
            vec![(d("2026-11-20"), "il mare".to_string())]
        );
        assert_eq!(s.tasks_done, vec!["Annaffiare".to_string()]);
        assert!(s.has_good_things());
    }

    #[test]
    fn trends_are_daily_means_with_gaps() {
        let events = vec![
            at(checkin(4, None), "2026-11-16", "09:00"),
            at(checkin(6, None), "2026-11-16", "19:00"),
            at(checkin(8, None), "2026-11-19", "09:00"),
            at(checkin(2, None), "2026-11-11", "09:00"), // previous week
        ];
        let s = weekly(&events, d("2026-11-16"), &tz());
        let mood = &s.trends[0];
        assert_eq!(mood.item, CheckinItem::Mood);
        assert_eq!(mood.days[0], Some(5.0));
        assert_eq!(mood.days[1], None);
        assert_eq!(mood.days[3], Some(8.0));
        assert_eq!(mood.mean, Some(6.5));
        assert_eq!(mood.previous_mean, Some(2.0));
        assert_eq!(s.trends[1].mean, None); // energy never answered
        assert_eq!(s.checkin_days, 2);
    }

    #[test]
    fn associations_need_enough_days_on_both_sides() {
        let mut events = vec![];
        // 8 days with a check-in; "Natura" on 4 of them.
        for (i, day) in [
            "2026-11-09",
            "2026-11-10",
            "2026-11-11",
            "2026-11-12",
            "2026-11-16",
            "2026-11-17",
            "2026-11-18",
            "2026-11-19",
        ]
        .iter()
        .enumerate()
        {
            let nature = i % 2 == 0;
            events.push(at(
                checkin(5, Some(if nature { 3 } else { 7 })),
                day,
                "20:00",
            ));
            if nature {
                events.push(at(care("Natura", None), day, "11:00"));
            }
        }
        events.push(at(care("Sonno", None), "2026-11-16", "08:00")); // 1 day only
        let s = weekly(&events, d("2026-11-16"), &tz());
        assert_eq!(s.associations.len(), 1);
        let a = &s.associations[0];
        assert_eq!(a.kind, FactorKind::Care);
        assert_eq!(a.label, "Natura");
        assert_eq!((a.days_with, a.days_without), (4, 4));
        assert_eq!(a.with[3], Some(3.0)); // mental load
        assert_eq!(a.without[3], Some(7.0));
        assert_eq!(a.with[0], Some(5.0));
    }

    #[test]
    fn an_empty_week_is_just_empty() {
        let s = weekly(&[], d("2026-11-16"), &tz());
        assert!(!s.has_good_things());
        assert_eq!(s.checkin_days, 0);
        assert!(s.trends.iter().all(|t| t.mean.is_none()));
        assert!(s.associations.is_empty());
    }
}
