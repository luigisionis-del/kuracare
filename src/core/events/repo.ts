import { migrations } from '../migrations';
import { EVENT_PREFIX, newEvent, type EventOrigin, type KuraEvent } from '../model/event';
import { isoWithOffset } from '../model/time';
import type { Vault } from '../store/vault';
import { KINDS, type Kind, type KindPayloads } from './kinds';

export type AnyEvent = { [K in Kind]: KuraEvent<K, KindPayloads[K]> }[Kind];
export type EventOf<K extends Kind> = KuraEvent<K, KindPayloads[K]>;

export const IMAGE_PREFIX = 'img:';

export async function addEvent<K extends Kind>(
  vault: Vault,
  kind: K,
  payload: KindPayloads[K],
  opts: { origin?: EventOrigin; when?: Date } = {},
): Promise<EventOf<K>> {
  const e = newEvent(kind, KINDS[kind].version, payload, opts.origin, opts.when);
  await vault.put(e.id, e);
  return e;
}

/** Saves a changed event; `editedAt` keeps track of after-the-fact changes. */
export async function updateEvent<E extends AnyEvent>(vault: Vault, e: E): Promise<E> {
  const u = { ...e, editedAt: isoWithOffset() };
  await vault.put(u.id, u);
  return u;
}

export async function removeEvent(vault: Vault, e: AnyEvent): Promise<void> {
  if (e.kind === 'meal' && e.payload.photoId) await vault.remove(e.payload.photoId);
  await vault.remove(e.id);
}

/** All events, upgraded to the current payload schema, newest first. */
export async function loadEvents(vault: Vault): Promise<AnyEvent[]> {
  const raw = await vault.list<KuraEvent>(EVENT_PREFIX);
  return raw.map((e) => migrations.upgrade(e) as AnyEvent).sort(byTimeDesc);
}

export function byTimeDesc(a: { ts: string }, b: { ts: string }): number {
  return Date.parse(b.ts) - Date.parse(a.ts);
}

export async function savePhoto(vault: Vault, dataUrl: string): Promise<string> {
  const id = `${IMAGE_PREFIX}${crypto.randomUUID()}`;
  await vault.put(id, { dataUrl });
  return id;
}

export async function loadPhoto(vault: Vault, id: string): Promise<string | undefined> {
  return (await vault.get<{ dataUrl: string }>(id))?.dataUrl;
}
