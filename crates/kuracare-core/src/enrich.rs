//! "Se ti va": factual details that can be added later, in a quiet moment.
//! Check-in answers are deliberately excluded: a mood rated hours later is a
//! memory, not a momentary assessment.

use crate::event::Event;
use crate::kinds::Payload;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    StrainValues,
    Amount,
    Route,
    Fulfilment,
    Duration,
    Activity,
    MealWhat,
}

pub const ENRICH_DAYS: i64 = 7;
pub const ENRICH_MAX: usize = 5;

pub fn missing_details(e: &Event) -> Vec<Detail> {
    let mut out = vec![];
    match &e.payload {
        Payload::Cannabis(c) => {
            if c.thc_pct.is_none() || c.cbd_pct.is_none() {
                out.push(Detail::StrainValues);
            }
            if c.amount.is_none() {
                out.push(Detail::Amount);
            }
            if c.route.is_none() {
                out.push(Detail::Route);
            }
        }
        Payload::Supplement(s) if s.amount.is_none() => out.push(Detail::Amount),
        Payload::Care(c) => {
            if c.fulfilment.is_none() {
                out.push(Detail::Fulfilment);
            }
            if c.duration_min.is_none() {
                out.push(Detail::Duration);
            }
        }
        Payload::Home(h) => {
            if h.activity.is_none() {
                out.push(Detail::Activity);
            }
            if h.duration_min.is_none() {
                out.push(Detail::Duration);
            }
        }
        Payload::Meal(m) if m.description.is_none() && m.photo_id.is_none() => {
            out.push(Detail::MealWhat)
        }
        _ => {}
    }
    out
}

/// At most `ENRICH_MAX` recent, not settled events, in the given (newest-first) order.
pub fn candidates(events: &[Event], now: jiff::Timestamp) -> Vec<&Event> {
    let since = now - jiff::SignedDuration::from_hours(24 * ENRICH_DAYS);
    events
        .iter()
        .filter(|e| !e.settled && e.timestamp() >= since && !missing_details(e).is_empty())
        .take(ENRICH_MAX)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::*;
    use crate::time;

    fn at(p: Payload, days_ago: i64) -> Event {
        let when = time::now()
            .checked_sub(jiff::SignedDuration::from_hours(24 * days_ago))
            .unwrap();
        Event::at(p, Origin::Spontaneous, when)
    }

    #[test]
    fn offers_factual_details_never_checkin_answers() {
        assert!(missing_details(&at(Payload::Checkin(Checkin::empty()), 0)).is_empty());
        let c = Cannabis {
            preset_id: None,
            strain: "X".into(),
            thc_pct: None,
            cbd_pct: Some(1.0),
            amount: Some(0.1),
            unit: "g".into(),
            route: Some(Route::Oral),
        };
        assert_eq!(
            missing_details(&at(Payload::Cannabis(c), 0)),
            vec![Detail::StrainValues]
        );
        let m = Meal {
            meal_type: MealType::Lunch,
            description: None,
            photo_id: None,
        };
        assert_eq!(
            missing_details(&at(Payload::Meal(m), 0)),
            vec![Detail::MealWhat]
        );
        let h = Home {
            room: "Cucina".into(),
            activity: None,
            duration_min: Some(15),
            fulfilment: None,
        };
        assert_eq!(
            missing_details(&at(Payload::Home(h), 0)),
            vec![Detail::Activity]
        );
    }

    #[test]
    fn only_recent_not_settled_at_most_five() {
        let care = |d, settled| {
            let mut e = at(
                Payload::Care(Care {
                    label: "X".into(),
                    duration_min: None,
                    fulfilment: None,
                }),
                d,
            );
            e.settled = settled;
            e
        };
        let list = vec![
            care(0, false),
            care(1, true),
            care(2, false),
            care(3, false),
            care(4, false),
            care(5, false),
            care(6, false),
            care(10, false),
        ];
        let out = candidates(&list, jiff::Timestamp::now());
        assert_eq!(out.len(), 5);
        assert!(out.iter().all(|e| !e.settled));
    }
}
