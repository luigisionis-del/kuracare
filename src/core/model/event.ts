import { isoWithOffset, deviceTimeZone } from './time';

/**
 * Every record is an event: an envelope plus a typed payload.
 *
 * Two different versions, never mix them up:
 *  - `schemaVersion`: storage shape of the payload for this `kind`.
 *    Old payloads are upgraded at read time by src/core/migrations.
 *  - instrument versions (e.g. check-in item set "core-v0.1", questionnaire
 *    "gad7@x"): live *inside* the payload and are never migrated — an answer
 *    given to core-v0.1 stays an answer to core-v0.1 forever.
 *
 * Missing answers are stored as null, never estimated.
 */
export const ENVELOPE_VERSION = 1;

export type EventOrigin = 'prompted' | 'spontaneous';

export interface KuraEvent<K extends string = string, P = unknown> {
  ev: typeof ENVELOPE_VERSION;
  id: string;
  /** When it happened (editable afterwards), ISO 8601 with offset. */
  ts: string;
  /** IANA zone at recording time. */
  tz: string;
  kind: K;
  origin: EventOrigin;
  schemaVersion: number;
  payload: P;
  /** When it was written, ISO 8601 with offset. */
  recordedAt: string;
  editedAt?: string;
  /** "Va bene così": never offered again for enrichment. */
  settled?: boolean;
}

export const EVENT_PREFIX = 'evt:';

export function newEvent<K extends string, P>(
  kind: K,
  schemaVersion: number,
  payload: P,
  origin: EventOrigin = 'spontaneous',
  when: Date = new Date(),
): KuraEvent<K, P> {
  const now = isoWithOffset();
  return {
    ev: ENVELOPE_VERSION,
    id: `${EVENT_PREFIX}${crypto.randomUUID()}`,
    ts: isoWithOffset(when),
    tz: deviceTimeZone(),
    kind,
    origin,
    schemaVersion,
    payload,
    recordedAt: now,
  };
}
