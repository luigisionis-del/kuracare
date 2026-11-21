import { migrations } from '../migrations';

/**
 * Event kinds and their payloads (schemaVersion 1).
 * Missing answers are null, never estimated.
 * `research` lists the only payload fields that may ever leave the device in a
 * research export (allow-list): free text and photos are never in it.
 */

export const CHECKIN_ITEM_SET = 'core-v0.1';
export const CHECKIN_ITEMS = ['mood', 'energy', 'pain', 'mentalLoad', 'anxiety'] as const;
export type CheckinItem = (typeof CHECKIN_ITEMS)[number];
export type CheckinPayload = { itemSet: typeof CHECKIN_ITEM_SET } & Record<CheckinItem, number | null>;

export interface ThoughtPayload {
  present: boolean;
  label: string | null;
  note: string | null;
}

export interface CarePayload {
  label: string;
  durationMin: number | null;
  /** 0-10, how fulfilling it was */
  fulfilment: number | null;
}

export interface ReflectionPayload {
  text: string;
}

export const ROUTES = ['inhalation', 'vaporization', 'oral', 'sublingual'] as const;
export type Route = (typeof ROUTES)[number];

export interface CannabisPayload {
  presetId: string | null;
  strain: string;
  thcPct: number | null;
  cbdPct: number | null;
  amount: number | null;
  unit: string;
  route: Route | null;
}

export interface SupplementPayload {
  presetId: string | null;
  name: string;
  amount: number | null;
  unit: string;
}

export const MEAL_TYPES = ['breakfast', 'lunch', 'dinner', 'snack'] as const;
export type MealType = (typeof MEAL_TYPES)[number];

export interface MealPayload {
  mealType: MealType;
  description: string | null;
  /** id of an `img:` record, kept on the device only */
  photoId: string | null;
}

export interface KindPayloads {
  checkin: CheckinPayload;
  thought: ThoughtPayload;
  care: CarePayload;
  reflection: ReflectionPayload;
  cannabis: CannabisPayload;
  supplement: SupplementPayload;
  meal: MealPayload;
}
export type Kind = keyof KindPayloads;

export const KINDS: Record<Kind, { version: number; research: readonly string[] }> = {
  checkin: { version: 1, research: ['itemSet', ...CHECKIN_ITEMS] },
  thought: { version: 1, research: ['present', 'label'] },
  care: { version: 1, research: ['label', 'durationMin', 'fulfilment'] },
  reflection: { version: 1, research: [] },
  cannabis: { version: 1, research: ['strain', 'thcPct', 'cbdPct', 'amount', 'unit', 'route'] },
  supplement: { version: 1, research: ['name', 'amount', 'unit'] },
  meal: { version: 1, research: ['mealType'] },
};

for (const [kind, def] of Object.entries(KINDS)) migrations.register(kind, def.version);

/** A 0-10 answer: integer in range, or null when skipped. */
export function score(v: unknown): number | null {
  if (v === null || v === undefined || v === '') return null;
  const n = Number(v);
  return Number.isFinite(n) ? Math.min(10, Math.max(0, Math.round(n))) : null;
}

/** Positive number or null (amounts, percentages, minutes). */
export function positive(v: unknown, max = Infinity): number | null {
  if (v === null || v === undefined || v === '') return null;
  const n = Number(String(v).replace(',', '.'));
  return Number.isFinite(n) && n >= 0 && n <= max ? n : null;
}

export function emptyCheckin(): CheckinPayload {
  return { itemSet: CHECKIN_ITEM_SET, mood: null, energy: null, pain: null, mentalLoad: null, anxiety: null };
}
