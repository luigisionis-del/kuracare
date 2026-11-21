//! Payload migrations, per event kind: `step(kind, n)` turns a version-n
//! payload into version n+1. Steps are pure functions with their own tests.
//! Stored data is never rewritten in place: events are upgraded when read.

use serde_json::Value;

use crate::error::{Error, Result};
use crate::kinds;

type Step = fn(Value) -> Value;

/// No kind has needed a migration yet. When one does, add it here:
/// `("care", 1) => Some(care_v1_to_v2)`.
fn step(kind: &str, from: u32) -> Option<Step> {
    let _ = (kind, from);
    None
}

pub fn upgrade(kind: &str, version: u32, payload: Value) -> Result<Value> {
    let latest = kinds::schema_version(kind)
        .ok_or_else(|| Error::Unsupported(format!("event kind {kind}")))?;
    upgrade_with(kind, version, latest, payload, step)
}

fn upgrade_with(
    kind: &str,
    version: u32,
    latest: u32,
    mut payload: Value,
    step: impl Fn(&str, u32) -> Option<Step>,
) -> Result<Value> {
    if version > latest {
        return Err(Error::Migration(format!(
            "{kind} v{version} is newer than this app (v{latest})"
        )));
    }
    for v in version..latest {
        let f = step(kind, v)
            .ok_or_else(|| Error::Migration(format!("missing {kind} v{v} -> v{}", v + 1)))?;
        payload = f(payload);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn demo(kind: &str, from: u32) -> Option<Step> {
        match (kind, from) {
            ("demo", 1) => Some(|mut p| {
                p["b"] = json!(0);
                p
            }),
            ("demo", 2) => Some(|mut p| {
                let a = p["a"].take();
                p.as_object_mut().unwrap().remove("a");
                p["renamed"] = a;
                p
            }),
            _ => None,
        }
    }

    #[test]
    fn upgrades_step_by_step_keeping_the_instrument_version() {
        let p = json!({"a": 5, "itemSet": "core-v0.1"});
        let up = upgrade_with("demo", 1, 3, p, demo).unwrap();
        assert_eq!(up, json!({"renamed": 5, "b": 0, "itemSet": "core-v0.1"}));
    }

    #[test]
    fn refuses_gaps_and_future_versions() {
        assert!(matches!(
            upgrade_with("demo", 1, 4, json!({"a": 1}), demo),
            Err(Error::Migration(_))
        ));
        assert!(matches!(
            upgrade("care", 9, json!({})),
            Err(Error::Migration(_))
        ));
        assert!(matches!(
            upgrade("unknown", 1, json!({})),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn current_kinds_have_no_gaps() {
        for (kind, latest) in kinds::KINDS {
            for v in 1..latest {
                assert!(step(kind, v).is_some(), "missing {kind} v{v}");
            }
        }
    }
}
