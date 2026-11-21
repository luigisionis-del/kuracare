//! Every record is an event: an envelope plus a typed payload.
//!
//! Two different versions, never mix them up:
//! - `schema_version`: storage shape of the payload for its kind. Old payloads
//!   are upgraded at read time (`migrations`), stored data is never rewritten.
//! - instrument versions (check-in item set `core-v0.1`, questionnaire ids):
//!   live inside the payload and are never migrated.
//!
//! Missing answers are `None`/null, never estimated.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::kinds::Payload;
use crate::migrations;
use crate::time;

pub const ENVELOPE_VERSION: u32 = 1;
pub const EVENT_PREFIX: &str = "evt:";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    /// Answer to an invitation from the app.
    Prompted,
    /// Recorded on the user's own initiative.
    Spontaneous,
}

/// Stored form: kind and payload stay untyped JSON until migrated.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEvent {
    pub ev: u32,
    pub id: String,
    pub ts: String,
    pub tz: String,
    pub kind: String,
    pub origin: Origin,
    pub schema_version: u32,
    pub payload: Value,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub settled: bool,
}

/// Typed, current-schema event used by the app.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub id: String,
    /// When it happened (editable afterwards).
    pub ts: String,
    pub tz: String,
    pub origin: Origin,
    pub payload: Payload,
    /// When it was written.
    pub recorded_at: String,
    pub edited_at: Option<String>,
    /// "Va bene così": never offered again for enrichment.
    pub settled: bool,
}

impl Event {
    pub fn new(payload: Payload, origin: Origin) -> Self {
        Self::at(payload, origin, time::now())
    }

    pub fn at(payload: Payload, origin: Origin, when: jiff::ZonedDateTime) -> Self {
        Self {
            id: format!("{EVENT_PREFIX}{}", uuid::Uuid::new_v4()),
            ts: time::format(&when),
            tz: time::zone_name(&when),
            origin,
            payload,
            recorded_at: time::now_iso(),
            edited_at: None,
            settled: false,
        }
    }

    pub fn kind(&self) -> &'static str {
        self.payload.kind()
    }

    pub fn to_raw(&self) -> Result<RawEvent> {
        Ok(RawEvent {
            ev: ENVELOPE_VERSION,
            id: self.id.clone(),
            ts: self.ts.clone(),
            tz: self.tz.clone(),
            kind: self.kind().into(),
            origin: self.origin,
            schema_version: self.payload.schema_version(),
            payload: self.payload.to_value()?,
            recorded_at: self.recorded_at.clone(),
            edited_at: self.edited_at.clone(),
            settled: self.settled,
        })
    }

    /// Upgrades the payload to the current schema, then types it.
    pub fn from_raw(raw: RawEvent) -> Result<Self> {
        let value = migrations::upgrade(&raw.kind, raw.schema_version, raw.payload)?;
        Ok(Self {
            payload: Payload::from_value(&raw.kind, value)?,
            id: raw.id,
            ts: raw.ts,
            tz: raw.tz,
            origin: raw.origin,
            recorded_at: raw.recorded_at,
            edited_at: raw.edited_at,
            settled: raw.settled,
        })
    }

    pub fn timestamp(&self) -> jiff::Timestamp {
        time::parse(&self.ts).unwrap_or(jiff::Timestamp::UNIX_EPOCH)
    }
}
