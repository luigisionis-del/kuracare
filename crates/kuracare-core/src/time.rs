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
