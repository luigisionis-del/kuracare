//! Research view of an event: only envelope basics and the allow-listed
//! payload fields. Built by inclusion, so new fields stay private by default.

use serde_json::{Map, Value};

use jiff::civil::Date;
use jiff::tz::TimeZone;

use crate::error::Result;
use crate::event::Event;
use crate::kinds::research_fields;

pub const RESEARCH_FORMAT: &str = "kuracare-research";
pub const RESEARCH_VERSION: u32 = 1;

/// Events whose local day is within `from..=to` (open ends allowed).
pub fn select<'a>(
    events: &'a [Event],
    from: Option<Date>,
    to: Option<Date>,
    tz: &TimeZone,
) -> Vec<&'a Event> {
    events
        .iter()
        .filter(|e| {
            let day = e.timestamp().to_zoned_datetime(tz.clone()).date();
            from.is_none_or(|f| day >= f) && to.is_none_or(|t| day <= t)
        })
        .collect()
}

/// Kinds with how many events, and the fields that leave for each.
pub fn preview(events: &[&Event]) -> Vec<(String, usize, &'static [&'static str])> {
    let mut counts: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for e in events {
        *counts.entry(e.kind()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(k, n)| (k.to_owned(), n, research_fields(k)))
        .collect()
}

/// Research export as JSON, oldest first.
pub fn research_json(events: &[&Event]) -> Result<Vec<u8>> {
    let mut sorted: Vec<&&Event> = events.iter().collect();
    sorted.sort_by_key(|e| e.timestamp());
    let rows = sorted
        .into_iter()
        .map(|e| research_view(e))
        .collect::<Result<Vec<_>>>()?;
    Ok(serde_json::to_vec_pretty(&serde_json::json!({
        "format": RESEARCH_FORMAT,
        "version": RESEARCH_VERSION,
        "exportedAt": crate::time::now_iso(),
        "events": rows,
    }))?)
}

/// Text that a spreadsheet would run as a formula gets a leading apostrophe
/// (OWASP "CSV injection"); labels are typed by the user.
fn defuse(s: &str) -> String {
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.to_owned()
    }
}

fn csv_cell(s: &str) -> String {
    let s = &defuse(s);
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// Research export as CSV in long form: one row per event and field.
/// Numbers and strings as they are, lists as JSON, missing as empty.
pub fn research_csv(events: &[&Event]) -> Result<String> {
    let mut sorted: Vec<&&Event> = events.iter().collect();
    sorted.sort_by_key(|e| e.timestamp());
    let mut out = String::from("ts,tz,kind,origin,schema_version,edited_after,field,value\n");
    for e in sorted {
        let v = research_view(e)?;
        let origin = v["origin"].as_str().unwrap_or_default();
        let fields = v["payload"].as_object().cloned().unwrap_or_default();
        for f in research_fields(e.kind()) {
            let value = match fields.get(*f) {
                None | Some(Value::Null) => String::new(),
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
            };
            out += &format!(
                "{},{},{},{},{},{},{},{}\n",
                csv_cell(v["ts"].as_str().unwrap_or_default()),
                csv_cell(v["tz"].as_str().unwrap_or_default()),
                e.kind(),
                origin,
                v["schemaVersion"],
                v["editedAfter"],
                f,
                csv_cell(&value),
            );
        }
    }
    Ok(out)
}

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

    #[test]
    fn csv_and_json_never_carry_free_text_and_escape_cells() {
        let tz = TimeZone::get("Europe/Rome").unwrap();
        let events = vec![
            Event::new(
                Payload::Thought(Thought {
                    present: true,
                    label: Some("Rimuginio, \"forte\"".into()),
                    note: Some("segreto".into()),
                }),
                Origin::Spontaneous,
            ),
            Event::new(
                Payload::Task(Task {
                    effort: Some(2),
                    ..Task::new("Chiamare Anna")
                }),
                Origin::Spontaneous,
            ),
        ];
        let sel = select(&events, None, None, &tz);
        let csv = research_csv(&sel).unwrap();
        assert!(!csv.contains("segreto") && !csv.contains("Anna"));
        assert!(csv.contains("\"Rimuginio, \"\"forte\"\"\""), "{csv}");
        assert!(csv.lines().any(|l| l.ends_with(",effort,2")));
        let formula = vec![Event::new(
            Payload::Care(Care {
                label: "=HYPERLINK(\"x\")".into(),
                duration_min: None,
                fulfilment: None,
            }),
            Origin::Spontaneous,
        )];
        let f = research_csv(&select(&formula, None, None, &tz)).unwrap();
        assert!(f.contains(",label,\"'=HYPERLINK(\"\"x\"\")\""), "{f}");
        let json = String::from_utf8(research_json(&sel).unwrap()).unwrap();
        assert!(!json.contains("segreto") && !json.contains("Anna"));
        assert!(json.contains(RESEARCH_FORMAT));
        let p = preview(&sel);
        assert_eq!(p.len(), 2);
        assert!(p.iter().all(|(_, n, _)| *n == 1));
        // A range in the past selects nothing.
        let past: Date = "2000-01-01".parse().unwrap();
        assert!(select(&events, None, Some(past), &tz).is_empty());
    }
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
