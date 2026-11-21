//! Versioned questionnaires (GAD-7, PHQ-8), defined in `content/questionnaires/`.
//!
//! Kuracare never writes item texts: the definition files hold placeholders
//! until Luigi provides the official Italian texts (after checking the licence).
//! A definition with `ready: false` is never shown. Scores come from the
//! definition file; no interpretation thresholds are shown anywhere.
//! Never PHQ-9 (its item on self-harm needs a response protocol and an ethics board).

use serde::{Deserialize, Serialize};

use crate::event::Event;
use crate::kinds::Payload;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerOption {
    pub label: String,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scoring {
    /// Sum of the item values; only when every item is answered.
    Sum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub id: String,
    /// Instrument version, stored with every answer and never migrated.
    pub version: String,
    pub name: String,
    pub ready: bool,
    pub every_days: u32,
    pub intro: String,
    pub items: Vec<String>,
    pub options: Vec<AnswerOption>,
    pub scoring: Scoring,
}

const BUILTIN: [&str; 2] = [
    include_str!("../../../content/questionnaires/gad7.it.json"),
    include_str!("../../../content/questionnaires/phq8.it.json"),
];

pub fn builtin() -> Vec<Definition> {
    BUILTIN
        .iter()
        .map(|s| serde_json::from_str(s).expect("valid questionnaire definition"))
        .collect()
}

/// The score per the definition, or None if any item is unanswered or invalid.
pub fn score(def: &Definition, answers: &[Option<u8>]) -> Option<u32> {
    if answers.len() != def.items.len() {
        return None;
    }
    let valid = |v: u8| def.options.iter().any(|o| o.value == v);
    match def.scoring {
        Scoring::Sum => answers
            .iter()
            .map(|a| a.filter(|v| valid(*v)).map(u32::from))
            .sum(),
    }
}

/// Ready questionnaires whose last answer is at least `every_days` old (or never).
pub fn due<'a>(
    defs: &'a [Definition],
    events: &[Event],
    now: jiff::Timestamp,
) -> Vec<&'a Definition> {
    defs.iter()
        .filter(|d| d.ready)
        .filter(|d| {
            let last = events
                .iter()
                .filter(|e| matches!(&e.payload, Payload::Questionnaire(q) if q.instrument == d.id))
                .map(Event::timestamp)
                .max();
            last.is_none_or(|t| now.duration_since(t).as_hours() >= i64::from(d.every_days) * 24)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::Questionnaire;

    #[test]
    fn builtin_definitions_are_placeholders_and_hidden() {
        let defs = builtin();
        assert_eq!(defs.len(), 2);
        let gad = defs.iter().find(|d| d.id == "gad7").unwrap();
        let phq = defs.iter().find(|d| d.id == "phq8").unwrap();
        assert_eq!(gad.items.len(), 7);
        assert_eq!(phq.items.len(), 8);
        for d in &defs {
            assert!(
                !d.ready,
                "{} must stay hidden until the official texts arrive",
                d.id
            );
            assert!(d.items.iter().all(|i| i.starts_with("[SEGNAPOSTO")));
            assert!(d.version.contains("placeholder"));
        }
        assert!(defs.iter().all(|d| !d.id.contains("phq9")));
    }

    #[test]
    fn scores_only_complete_answers() {
        let def = &builtin()[0];
        let all = vec![Some(1); 7];
        assert_eq!(score(def, &all), Some(7));
        let mut missing = all.clone();
        missing[3] = None;
        assert_eq!(score(def, &missing), None);
        let mut invalid = all.clone();
        invalid[0] = Some(9);
        assert_eq!(score(def, &invalid), None);
        assert_eq!(score(def, &all[..6]), None);
    }

    #[test]
    fn due_every_n_days_and_only_when_ready() {
        let mut defs = builtin();
        let now = jiff::Timestamp::now();
        assert!(due(&defs, &[], now).is_empty()); // nothing ready
        defs[0].ready = true;
        assert_eq!(due(&defs, &[], now).len(), 1);
        let answered = Event::new(
            Payload::Questionnaire(Questionnaire {
                instrument: "gad7".into(),
                version: defs[0].version.clone(),
                answers: vec![Some(0); 7],
                score: Some(0),
            }),
            Origin::Prompted,
        );
        assert!(due(&defs, &[answered], now).is_empty());
        let later = now + jiff::SignedDuration::from_hours(24 * 14);
        assert_eq!(due(&defs, &[], later).len(), 1);
    }
}
