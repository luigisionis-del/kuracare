//! Event persistence on top of the vault.

use crate::error::Result;
use crate::event::{EVENT_PREFIX, Event, RawEvent};
use crate::kinds::Payload;
use crate::store::Store;
use crate::time;
use crate::vault::Vault;

pub const IMAGE_PREFIX: &str = "img:";

pub fn save<S: Store>(v: &Vault<S>, e: &Event) -> Result<()> {
    v.put(&e.id, &e.to_raw()?)
}

/// Saves a changed event; `edited_at` records after-the-fact changes.
pub fn update<S: Store>(v: &Vault<S>, e: &Event) -> Result<Event> {
    let mut u = e.clone();
    u.edited_at = Some(time::now_iso());
    save(v, &u)?;
    Ok(u)
}

pub fn remove<S: Store>(v: &Vault<S>, e: &Event) -> Result<()> {
    if let Payload::Meal(m) = &e.payload
        && let Some(photo) = &m.photo_id
    {
        v.remove(photo)?;
    }
    v.remove(&e.id)
}

/// All events, upgraded to the current schemas, newest first.
pub fn load<S: Store>(v: &Vault<S>) -> Result<Vec<Event>> {
    let raw: Vec<RawEvent> = v.list(EVENT_PREFIX)?;
    let mut events = raw
        .into_iter()
        .map(Event::from_raw)
        .collect::<Result<Vec<_>>>()?;
    sort_newest_first(&mut events);
    Ok(events)
}

pub fn sort_newest_first(events: &mut [Event]) {
    events.sort_by_key(|e| std::cmp::Reverse(e.timestamp()));
}

/// Photos are stored as their own sealed records (JPEG bytes).
pub fn save_photo<S: Store>(v: &Vault<S>, jpeg: &[u8]) -> Result<String> {
    let id = format!("{IMAGE_PREFIX}{}", uuid::Uuid::new_v4());
    v.put(&id, &jpeg.to_vec())?;
    Ok(id)
}

pub fn load_photo<S: Store>(v: &Vault<S>, id: &str) -> Result<Option<Vec<u8>>> {
    v.get(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::kinds::{Care, Checkin, Meal, MealType};
    use crate::vault::test_vault;
    use serde_json::json;

    #[test]
    fn add_load_update_remove() {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        let earlier = time::now()
            .checked_sub(jiff::SignedDuration::from_mins(1))
            .unwrap();
        let a = Event::at(
            Payload::Checkin(Checkin {
                mood: Some(6),
                ..Checkin::empty()
            }),
            Origin::Spontaneous,
            earlier,
        );
        let b = Event::new(
            Payload::Care(Care {
                label: "Passeggiata".into(),
                duration_min: None,
                fulfilment: None,
            }),
            Origin::Spontaneous,
        );
        save(&v, &a).unwrap();
        save(&v, &b).unwrap();
        let all = load(&v).unwrap();
        assert_eq!(
            all.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec![b.id.as_str(), a.id.as_str()]
        );

        let mut changed = b.clone();
        if let Payload::Care(c) = &mut changed.payload {
            c.fulfilment = Some(8);
        }
        let u = update(&v, &changed).unwrap();
        assert!(u.edited_at.is_some());
        assert_eq!(u.recorded_at, b.recorded_at);
        assert_eq!(load(&v).unwrap()[0].payload, changed.payload);

        remove(&v, &a).unwrap();
        assert_eq!(load(&v).unwrap().len(), 1);
    }

    #[test]
    fn removing_a_meal_removes_its_photo() {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        let photo = save_photo(&v, b"\xff\xd8jpeg").unwrap();
        let m = Event::new(
            Payload::Meal(Meal {
                meal_type: MealType::Lunch,
                description: None,
                photo_id: Some(photo.clone()),
            }),
            Origin::Spontaneous,
        );
        save(&v, &m).unwrap();
        assert_eq!(load_photo(&v, &photo).unwrap().unwrap(), b"\xff\xd8jpeg");
        remove(&v, &m).unwrap();
        assert!(load_photo(&v, &photo).unwrap().is_none());
    }

    #[test]
    fn reads_events_written_by_the_pwa_format() {
        // Same JSON shape as the Svelte PWA (tag pwa-final).
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        let raw = json!({
            "ev": 1, "id": "evt:x", "ts": "2026-11-21T09:30:00.000+01:00", "tz": "Europe/Rome",
            "kind": "care", "origin": "spontaneous", "schemaVersion": 1,
            "payload": {"label": "Natura", "durationMin": 20, "fulfilment": null},
            "recordedAt": "2026-11-21T09:31:00.000+01:00"
        });
        v.put("evt:x", &raw).unwrap();
        let e = &load(&v).unwrap()[0];
        assert_eq!(
            e.payload,
            Payload::Care(Care {
                label: "Natura".into(),
                duration_min: Some(20),
                fulfilment: None
            })
        );
        assert!(!e.settled);
    }
}
