//! Event kinds and their payloads. Missing answers are `None` (null in JSON).
//! `research_fields` lists the only payload fields that may ever leave the
//! device in a research export (allow-list): free text and photos never do.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

pub const CHECKIN_ITEM_SET: &str = "core-v0.1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkin {
    /// Instrument version (never migrated).
    pub item_set: String,
    pub mood: Option<u8>,
    pub energy: Option<u8>,
    pub pain: Option<u8>,
    pub mental_load: Option<u8>,
    pub anxiety: Option<u8>,
}

impl Checkin {
    pub fn empty() -> Self {
        Self {
            item_set: CHECKIN_ITEM_SET.into(),
            ..Default::default()
        }
    }
    pub fn answered(&self) -> usize {
        [
            self.mood,
            self.energy,
            self.pain,
            self.mental_load,
            self.anxiety,
        ]
        .iter()
        .filter(|v| v.is_some())
        .count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckinItem {
    Mood,
    Energy,
    Pain,
    MentalLoad,
    Anxiety,
}

impl CheckinItem {
    pub const ALL: [CheckinItem; 5] = [
        Self::Mood,
        Self::Energy,
        Self::Pain,
        Self::MentalLoad,
        Self::Anxiety,
    ];

    pub fn get(self, c: &Checkin) -> Option<u8> {
        match self {
            Self::Mood => c.mood,
            Self::Energy => c.energy,
            Self::Pain => c.pain,
            Self::MentalLoad => c.mental_load,
            Self::Anxiety => c.anxiety,
        }
    }

    pub fn set(self, c: &mut Checkin, v: Option<u8>) {
        let v = v.map(|n| n.min(10));
        match self {
            Self::Mood => c.mood = v,
            Self::Energy => c.energy = v,
            Self::Pain => c.pain = v,
            Self::MentalLoad => c.mental_load = v,
            Self::Anxiety => c.anxiety = v,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thought {
    pub present: bool,
    pub label: Option<String>,
    /// Free text: device only.
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Care {
    pub label: String,
    pub duration_min: Option<u32>,
    /// 0-10, how fulfilling it was.
    pub fulfilment: Option<u8>,
}

/// Looking after the home: a room, optionally what was done.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub room: String,
    pub activity: Option<String>,
    pub duration_min: Option<u32>,
    pub fulfilment: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reflection {
    /// Free text: device only, never exported for research.
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Route {
    Inhalation,
    Vaporization,
    Oral,
    Sublingual,
}

impl Route {
    pub const ALL: [Route; 4] = [
        Self::Inhalation,
        Self::Vaporization,
        Self::Oral,
        Self::Sublingual,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cannabis {
    pub preset_id: Option<String>,
    pub strain: String,
    pub thc_pct: Option<f64>,
    pub cbd_pct: Option<f64>,
    pub amount: Option<f64>,
    pub unit: String,
    pub route: Option<Route>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplement {
    pub preset_id: Option<String>,
    pub name: String,
    pub amount: Option<f64>,
    pub unit: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl MealType {
    pub const ALL: [MealType; 4] = [Self::Breakfast, Self::Lunch, Self::Dinner, Self::Snack];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meal {
    pub meal_type: MealType,
    /// Free text: device only.
    pub description: Option<String>,
    /// Id of an `img:` record, device only.
    pub photo_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    Checkin(Checkin),
    Thought(Thought),
    Care(Care),
    Home(Home),
    Reflection(Reflection),
    Cannabis(Cannabis),
    Supplement(Supplement),
    Meal(Meal),
}

/// Every kind with its current schema version.
pub const KINDS: [(&str, u32); 8] = [
    ("checkin", 1),
    ("thought", 1),
    ("care", 1),
    ("home", 1),
    ("reflection", 1),
    ("cannabis", 1),
    ("supplement", 1),
    ("meal", 1),
];

/// Allow-list for research exports. Never free text, photos or preset ids.
pub fn research_fields(kind: &str) -> &'static [&'static str] {
    match kind {
        "checkin" => &["itemSet", "mood", "energy", "pain", "mentalLoad", "anxiety"],
        "thought" => &["present", "label"],
        "care" => &["label", "durationMin", "fulfilment"],
        "home" => &["room", "activity", "durationMin", "fulfilment"],
        "cannabis" => &["strain", "thcPct", "cbdPct", "amount", "unit", "route"],
        "supplement" => &["name", "amount", "unit"],
        "meal" => &["mealType"],
        _ => &[],
    }
}

impl Payload {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Checkin(_) => "checkin",
            Self::Thought(_) => "thought",
            Self::Care(_) => "care",
            Self::Home(_) => "home",
            Self::Reflection(_) => "reflection",
            Self::Cannabis(_) => "cannabis",
            Self::Supplement(_) => "supplement",
            Self::Meal(_) => "meal",
        }
    }

    pub fn schema_version(&self) -> u32 {
        schema_version(self.kind()).expect("every payload kind is listed in KINDS")
    }

    pub fn to_value(&self) -> Result<Value> {
        Ok(match self {
            Self::Checkin(p) => serde_json::to_value(p)?,
            Self::Thought(p) => serde_json::to_value(p)?,
            Self::Care(p) => serde_json::to_value(p)?,
            Self::Home(p) => serde_json::to_value(p)?,
            Self::Reflection(p) => serde_json::to_value(p)?,
            Self::Cannabis(p) => serde_json::to_value(p)?,
            Self::Supplement(p) => serde_json::to_value(p)?,
            Self::Meal(p) => serde_json::to_value(p)?,
        })
    }

    pub fn from_value(kind: &str, v: Value) -> Result<Self> {
        Ok(match kind {
            "checkin" => Self::Checkin(serde_json::from_value(v)?),
            "thought" => Self::Thought(serde_json::from_value(v)?),
            "care" => Self::Care(serde_json::from_value(v)?),
            "home" => Self::Home(serde_json::from_value(v)?),
            "reflection" => Self::Reflection(serde_json::from_value(v)?),
            "cannabis" => Self::Cannabis(serde_json::from_value(v)?),
            "supplement" => Self::Supplement(serde_json::from_value(v)?),
            "meal" => Self::Meal(serde_json::from_value(v)?),
            other => return Err(Error::Unsupported(format!("event kind {other}"))),
        })
    }
}

pub fn schema_version(kind: &str) -> Option<u32> {
    KINDS.iter().find(|(k, _)| *k == kind).map(|(_, v)| *v)
}

/// A 0-10 answer from text: integer in range, or None when skipped/invalid.
pub fn score(text: &str) -> Option<u8> {
    let n: f64 = text.trim().replace(',', ".").parse().ok()?;
    n.is_finite().then(|| n.round().clamp(0.0, 10.0) as u8)
}

/// Non-negative number from text, accepting Italian decimal commas.
pub fn positive(text: &str, max: f64) -> Option<f64> {
    let n: f64 = text.trim().replace(',', ".").parse().ok()?;
    (n.is_finite() && (0.0..=max).contains(&n)).then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_and_positive() {
        assert_eq!(score("7"), Some(7));
        assert_eq!(score("0"), Some(0));
        assert_eq!(score("12"), Some(10));
        assert_eq!(score(""), None);
        assert_eq!(score("x"), None);
        assert_eq!(positive("0,5", f64::INFINITY), Some(0.5));
        assert_eq!(positive("-1", f64::INFINITY), None);
        assert_eq!(positive("150", 100.0), None);
    }

    #[test]
    fn research_allow_list_never_has_free_text_or_photos() {
        for (kind, _) in KINDS {
            for forbidden in ["note", "text", "description", "photoId", "presetId"] {
                assert!(
                    !research_fields(kind).contains(&forbidden),
                    "{kind}.{forbidden}"
                );
            }
        }
        assert!(research_fields("reflection").is_empty());
    }

    #[test]
    fn missing_answers_serialise_as_null() {
        let v = Payload::Checkin(Checkin {
            mood: Some(6),
            ..Checkin::empty()
        })
        .to_value()
        .unwrap();
        assert_eq!(v["mood"], 6);
        assert!(v["energy"].is_null());
        assert_eq!(v["itemSet"], "core-v0.1");
    }

    #[test]
    fn every_kind_round_trips() {
        let samples = [
            Payload::Checkin(Checkin::empty()),
            Payload::Thought(Thought {
                present: true,
                label: Some("Rimuginio".into()),
                note: None,
            }),
            Payload::Care(Care {
                label: "Natura".into(),
                duration_min: Some(20),
                fulfilment: None,
            }),
            Payload::Home(Home {
                room: "Cucina".into(),
                activity: Some("Piatti".into()),
                duration_min: None,
                fulfilment: Some(7),
            }),
            Payload::Reflection(Reflection {
                text: "il sole".into(),
            }),
            Payload::Cannabis(Cannabis {
                preset_id: None,
                strain: "X".into(),
                thc_pct: None,
                cbd_pct: Some(12.5),
                amount: Some(0.1),
                unit: "g".into(),
                route: Some(Route::Vaporization),
            }),
            Payload::Supplement(Supplement {
                preset_id: None,
                name: "Glicina".into(),
                amount: None,
                unit: "mg".into(),
            }),
            Payload::Meal(Meal {
                meal_type: MealType::Lunch,
                description: None,
                photo_id: None,
            }),
        ];
        assert_eq!(samples.len(), KINDS.len());
        for p in samples {
            let back = Payload::from_value(p.kind(), p.to_value().unwrap()).unwrap();
            assert_eq!(back, p);
        }
    }
}
