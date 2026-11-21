//! Research view of an event: only envelope basics and the allow-listed
//! payload fields. Built by inclusion, so new fields stay private by default.

use serde_json::{Map, Value};

use crate::error::Result;
use crate::event::Event;
use crate::kinds::research_fields;

pub fn research_view(e: &Event) -> Result<Value> {
    let raw = e.to_raw()?;
    let mut payload = Map::new();
    for f in research_fields(&raw.kind) {
        payload.insert(
            (*f).to_owned(),
            raw.payload.get(*f).cloned().unwrap_or(Value::Null),
        );
    }
    Ok(serde_json::json!({
        "ts": raw.ts,
        "tz": raw.tz,
        "kind": raw.kind,
        "origin": raw.origin,
        "schemaVersion": raw.schema_version,
        "editedAfter": raw.edited_at.is_some(),
        "payload": payload,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::*;

    #[test]
    fn free_text_and_photos_never_leave() {
        let events = [
            Payload::Thought(Thought {
                present: true,
                label: Some("Rimuginio".into()),
                note: Some("SEGRETO".into()),
            }),
            Payload::Reflection(Reflection {
                text: "SEGRETO".into(),
            }),
            Payload::Meal(Meal {
                meal_type: MealType::Dinner,
                description: Some("SEGRETO".into()),
                photo_id: Some("img:SEGRETO".into()),
            }),
        ];
        for p in events {
            let v = research_view(&Event::new(p, Origin::Spontaneous)).unwrap();
            let s = v.to_string();
            assert!(!s.contains("SEGRETO"), "{s}");
            assert!(v.get("id").is_none(), "record ids stay on the device");
        }
    }

    #[test]
    fn keeps_allowed_fields_and_nulls() {
        let e = Event::new(
            Payload::Checkin(Checkin {
                mood: Some(7),
                ..Checkin::empty()
            }),
            Origin::Prompted,
        );
        let v = research_view(&e).unwrap();
        assert_eq!(v["payload"]["mood"], 7);
        assert!(v["payload"]["anxiety"].is_null());
        assert_eq!(v["payload"]["itemSet"], "core-v0.1");
        assert_eq!(v["origin"], "prompted");
    }
}
