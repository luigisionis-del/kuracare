//! Italian wording. The check-in labels and anchors are PROVISIONAL (core-v0.1):
//! they will be replaced by items chosen from the ESM Item Repository (https://osf.io/kg376).

use kuracare_core::kinds::{CheckinItem, MealType, Payload, Route};
use kuracare_core::{Event, time};

pub fn checkin_label(i: CheckinItem) -> (&'static str, &'static str, &'static str) {
    match i {
        CheckinItem::Mood => ("Umore", "giù", "bene"),
        CheckinItem::Energy => ("Energia", "scarica", "piena"),
        CheckinItem::Pain => ("Dolore", "nessuno", "molto"),
        CheckinItem::MentalLoad => ("Pesantezza mentale", "leggera", "pesante"),
        CheckinItem::Anxiety => ("Ansia", "calma", "molta"),
    }
}

pub fn route_label(r: Route) -> &'static str {
    match r {
        Route::Inhalation => "Inalazione",
        Route::Vaporization => "Vaporizzazione",
        Route::Oral => "Orale",
        Route::Sublingual => "Sublinguale",
    }
}

pub fn meal_label(m: MealType) -> &'static str {
    match m {
        MealType::Breakfast => "Colazione",
        MealType::Lunch => "Pranzo",
        MealType::Dinner => "Cena",
        MealType::Snack => "Spuntino",
    }
}

/// Number with an Italian decimal comma, empty when missing.
pub fn num(n: Option<f64>) -> String {
    n.map(|v| {
        let s = format!("{v}");
        s.replace('.', ",")
    })
    .unwrap_or_default()
}

fn join(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ")
}

fn amount(a: Option<f64>, unit: &str) -> String {
    a.map(|_| format!("{} {unit}", num(a))).unwrap_or_default()
}

fn minutes(m: Option<u32>) -> String {
    m.map(|m| format!("{m} min")).unwrap_or_default()
}

fn fulfilment(f: Option<u8>) -> String {
    f.map(|f| format!("appagamento {f}")).unwrap_or_default()
}

/// One short line describing an event, for lists.
pub fn describe(e: &Event) -> String {
    match &e.payload {
        Payload::Checkin(c) => match c.answered() {
            0 => "Check-in".into(),
            n => format!("Check-in ({n} risposte su 5)"),
        },
        Payload::Care(c) => join(&[
            c.label.clone(),
            minutes(c.duration_min),
            fulfilment(c.fulfilment),
        ]),
        Payload::Home(h) => join(&[
            h.room.clone(),
            h.activity
                .clone()
                .map(|a| a.to_lowercase())
                .unwrap_or_default(),
            minutes(h.duration_min),
            fulfilment(h.fulfilment),
        ]),
        Payload::Thought(t) => t
            .label
            .as_ref()
            .map(|l| format!("Pensiero: {}", l.to_lowercase()))
            .unwrap_or_else(|| "Un pensiero che torna".into()),
        Payload::Reflection(_) => "Cosa ti ha fatto bene".into(),
        Payload::Cannabis(c) => join(&[
            c.strain.clone(),
            amount(c.amount, &c.unit),
            c.route
                .map(|r| route_label(r).to_lowercase())
                .unwrap_or_default(),
        ]),
        Payload::Supplement(s) => join(&[s.name.clone(), amount(s.amount, &s.unit)]),
        Payload::Task(t) => t.title.clone(),
        Payload::Meal(m) => join(&[
            meal_label(m.meal_type).into(),
            m.description.clone().unwrap_or_else(|| {
                if m.photo_id.is_some() {
                    "foto".into()
                } else {
                    String::new()
                }
            }),
        ]),
    }
}

pub fn greeting(hour: i8) -> &'static str {
    match hour {
        0..=4 => "Buonanotte",
        5..=12 => "Buongiorno",
        13..=17 => "Buon pomeriggio",
        _ => "Buonasera",
    }
}

const WEEKDAYS: [&str; 7] = [
    "lunedì",
    "martedì",
    "mercoledì",
    "giovedì",
    "venerdì",
    "sabato",
    "domenica",
];
const MONTHS: [&str; 12] = [
    "gennaio",
    "febbraio",
    "marzo",
    "aprile",
    "maggio",
    "giugno",
    "luglio",
    "agosto",
    "settembre",
    "ottobre",
    "novembre",
    "dicembre",
];

/// "sabato 21 novembre"
pub fn long_date(z: &jiff::ZonedDateTime) -> String {
    let wd = WEEKDAYS[z.weekday().to_monday_zero_offset() as usize];
    format!("{wd} {} {}", z.day(), MONTHS[(z.month() - 1) as usize])
}

/// "sabato 21"
pub fn short_date(z: &jiff::ZonedDateTime) -> String {
    let wd = WEEKDAYS[z.weekday().to_monday_zero_offset() as usize];
    format!("{wd} {}", z.day())
}

/// "YYYY-MM-DD" -> "martedì 25 novembre" (as given if not a date).
pub fn day_label(day: &str) -> String {
    match day.parse::<jiff::civil::Date>() {
        Ok(d) => format!(
            "{} {} {}",
            WEEKDAYS[d.weekday().to_monday_zero_offset() as usize],
            d.day(),
            MONTHS[(d.month() - 1) as usize]
        ),
        Err(_) => day.to_owned(),
    }
}

/// A pause that ends in this year or later means "until I remove it".
pub const OPEN_PAUSE_YEAR: i16 = 2100;

/// "fino a martedì 25 novembre alle 09:00", or "finché non la togli".
pub fn pause_label(until: Option<&str>) -> String {
    let Some(z) = until.and_then(|u| time::local(u).ok()) else {
        return String::new();
    };
    // A year of slack: in any time zone the open pause starts on 1/1/2100 UTC.
    if z.year() >= OPEN_PAUSE_YEAR - 1 {
        return "finché non la togli".into();
    }
    let day = z.strftime("%Y-%m-%d").to_string();
    format!("fino a {} alle {}", day_label(&day), z.strftime("%H:%M"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuracare_core::Origin;
    use kuracare_core::kinds::*;

    #[test]
    fn describes_events_in_italian() {
        let e = Event::new(
            Payload::Cannabis(Cannabis {
                preset_id: None,
                strain: "Lemon Cockies".into(),
                thc_pct: None,
                cbd_pct: None,
                amount: Some(0.15),
                unit: "g".into(),
                route: Some(Route::Oral),
            }),
            Origin::Spontaneous,
        );
        assert_eq!(describe(&e), "Lemon Cockies · 0,15 g · orale");
        let h = Event::new(
            Payload::Home(Home {
                room: "Cucina".into(),
                activity: Some("Piatti".into()),
                duration_min: Some(15),
                fulfilment: None,
            }),
            Origin::Spontaneous,
        );
        assert_eq!(describe(&h), "Cucina · piatti · 15 min");
        let c = Event::new(
            Payload::Checkin(Checkin {
                mood: Some(7),
                anxiety: Some(3),
                ..Checkin::empty()
            }),
            Origin::Spontaneous,
        );
        assert_eq!(describe(&c), "Check-in (2 risposte su 5)");
    }

    #[test]
    fn dates_in_italian() {
        let z: jiff::ZonedDateTime = "2026-11-21T09:30:00+01:00[Europe/Rome]".parse().unwrap();
        assert_eq!(long_date(&z), "sabato 21 novembre");
        assert_eq!(greeting(9), "Buongiorno");
        assert_eq!(greeting(20), "Buonasera");
        assert_eq!(day_label("2026-11-25"), "mercoledì 25 novembre");
        assert_eq!(
            pause_label(Some("2100-01-01T00:00:00.000+01:00")),
            "finché non la togli"
        );
        assert!(
            pause_label(Some("2026-11-24T09:00:00.000+01:00")).starts_with("fino a martedì 24")
        );
    }
}
