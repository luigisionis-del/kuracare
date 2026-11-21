use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::store::Store;
use crate::vault::Vault;

pub const SETTINGS_ID: &str = "cfg:settings";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeWindow {
    /// "HH:MM", local time
    pub start: String,
    pub end: String,
}

impl TimeWindow {
    pub fn is_valid(&self) -> bool {
        fn hhmm(s: &str) -> bool {
            let b = s.as_bytes();
            b.len() == 5
                && b[2] == b':'
                && s[..2].parse::<u8>().is_ok_and(|h| h < 24)
                && s[3..].parse::<u8>().is_ok_and(|m| m < 60)
                && b[..2].iter().chain(&b[3..]).all(u8::is_ascii_digit)
        }
        hhmm(&self.start) && hhmm(&self.end) && self.start < self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub checkin_windows: Vec<TimeWindow>,
    pub lock_after_minutes: u32,
    /// While in the future, the app asks nothing.
    pub pause_until: Option<String>,
    /// Local date (YYYY-MM-DD) when the evening meal question got a "Non ora".
    pub meal_recall_dismissed_on: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            checkin_windows: vec![],
            lock_after_minutes: 5,
            pause_until: None,
            meal_recall_dismissed_on: None,
        }
    }
}

pub fn load<S: Store>(v: &Vault<S>) -> Result<Settings> {
    Ok(v.get(SETTINGS_ID)?.unwrap_or_default())
}

pub fn save<S: Store>(v: &Vault<S>, s: &Settings) -> Result<()> {
    v.put(SETTINGS_ID, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_time_windows() {
        let w = |a: &str, b: &str| TimeWindow {
            start: a.into(),
            end: b.into(),
        };
        assert!(w("09:00", "11:00").is_valid());
        assert!(!w("11:00", "09:00").is_valid());
        assert!(!w("9:00", "11:00").is_valid());
        assert!(!w("24:00", "25:00").is_valid());
    }
}
