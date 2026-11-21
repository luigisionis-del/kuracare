//! Timestamps are ISO 8601 local time with explicit offset,
//! e.g. `2026-11-21T09:30:00.000+01:00`, plus the IANA zone name.

use jiff::{Timestamp, ZonedDateTime, tz::TimeZone};

use crate::error::{Error, Result};

const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3f%:z";

pub fn format(z: &ZonedDateTime) -> String {
    z.strftime(FORMAT).to_string()
}

pub fn now() -> ZonedDateTime {
    ZonedDateTime::now()
}

pub fn now_iso() -> String {
    format(&now())
}

/// IANA name of the device zone ("Europe/Rome"), or the fixed offset if unknown.
pub fn zone_name(z: &ZonedDateTime) -> String {
    z.time_zone()
        .iana_name()
        .map(str::to_owned)
        .unwrap_or_else(|| z.offset().to_string())
}

pub fn parse(iso: &str) -> Result<Timestamp> {
    iso.parse::<Timestamp>()
        .map_err(|e| Error::Unsupported(format!("timestamp {iso}: {e}")))
}

/// The instant of `iso`, seen in the device zone.
pub fn local(iso: &str) -> Result<ZonedDateTime> {
    Ok(parse(iso)?.to_zoned_datetime(TimeZone::system()))
}

/// "HH:MM" of `iso` in the device zone.
pub fn hhmm(iso: &str) -> String {
    local(iso)
        .map(|z| z.strftime("%H:%M").to_string())
        .unwrap_or_default()
}

/// Local calendar day "YYYY-MM-DD" of `iso` (device zone).
pub fn local_day(iso: &str) -> String {
    local(iso)
        .map(|z| z.strftime("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub fn today() -> String {
    now().strftime("%Y-%m-%d").to_string()
}

/// Value for `<input type="datetime-local">` (device zone).
pub fn to_local_input(iso: &str) -> String {
    local(iso)
        .map(|z| z.strftime("%Y-%m-%dT%H:%M").to_string())
        .unwrap_or_default()
}

/// `<input type="datetime-local">` value -> ISO with offset, or None if invalid.
pub fn from_local_input(value: &str) -> Option<String> {
    let dt: jiff::civil::DateTime = value.parse().ok()?;
    dt.to_zoned_datetime(TimeZone::system())
        .ok()
        .map(|z| format(&z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_with_explicit_offset() {
        let s = now_iso();
        let re = |c: char| c.is_ascii_digit();
        assert_eq!(s.len(), "2026-11-21T09:30:00.000+01:00".len(), "{s}");
        assert!(s[..4].chars().all(re));
        assert!(matches!(&s[23..24], "+" | "-"), "{s}");
    }

    #[test]
    fn datetime_local_round_trip() {
        let iso = from_local_input("2026-11-21T09:30").unwrap();
        assert_eq!(to_local_input(&iso), "2026-11-21T09:30");
        assert_eq!(hhmm(&iso), "09:30");
        assert_eq!(local_day(&iso), "2026-11-21");
        assert!(from_local_input("nonsense").is_none());
    }

    #[test]
    fn round_trips_the_same_instant_around_dst() {
        let tz = TimeZone::get("Europe/Rome").unwrap();
        for t in [
            "2026-03-29T00:30:00Z",
            "2026-03-29T01:30:00Z",
            "2026-10-25T00:30:00Z",
            "2026-10-25T01:30:00Z",
        ] {
            let ts: Timestamp = t.parse().unwrap();
            let z = ts.to_zoned_datetime(tz.clone());
            assert_eq!(parse(&format(&z)).unwrap(), ts);
        }
    }
}
