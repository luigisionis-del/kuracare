import phrasesText from '../../content/riconoscimenti.it.txt?raw';
import type { EventOrigin } from '../core/model/event';
import type { Kind, KindPayloads } from '../core/events/kinds';
import { defaultLists, loadLists, saveLists, type Lists } from '../core/events/lists';
import { addEvent, byTimeDesc, loadEvents, removeEvent, updateEvent, type AnyEvent, type EventOf } from '../core/events/repo';
import { parsePhrases, pickPhrase } from '../core/events/recognition';
import { vault } from './vault';

const PHRASES = parsePhrases(phrasesText);

export interface Toast {
  text: string;
  /** Optional follow-up, e.g. open the event to add details. */
  action?: { label: string; eventId: string };
}

export const diary = $state<{ events: AnyEvent[]; lists: Lists; toast: Toast | null }>({
  events: [],
  lists: defaultLists(),
  toast: null,
});

let lastPhrase: string | null = null;
let toastTimer: number | undefined;

export async function loadDiary(): Promise<void> {
  diary.events = await loadEvents(vault);
  diary.lists = await loadLists(vault);
}

export function clearDiary(): void {
  diary.events = [];
  diary.lists = defaultLists();
  diary.toast = null;
}

export function findEvent(id: string): AnyEvent | undefined {
  return diary.events.find((e) => e.id === id);
}

export async function record<K extends Kind>(
  kind: K,
  payload: KindPayloads[K],
  opts: { origin?: EventOrigin; when?: Date } = {},
): Promise<EventOf<K>> {
  const e = await addEvent(vault, kind, payload, opts);
  diary.events = [e as AnyEvent, ...diary.events].sort(byTimeDesc);
  return e;
}

export async function change(e: AnyEvent): Promise<void> {
  const saved = await updateEvent(vault, $state.snapshot(e) as AnyEvent);
  diary.events = diary.events.map((x) => (x.id === saved.id ? saved : x)).sort(byTimeDesc);
}

export async function forget(e: AnyEvent): Promise<void> {
  await removeEvent(vault, $state.snapshot(e) as AnyEvent);
  diary.events = diary.events.filter((x) => x.id !== e.id);
}

export async function setLists(lists: Lists): Promise<void> {
  await saveLists(vault, lists);
  diary.lists = lists;
}

export function showToast(toast: Toast, ms = 6000): void {
  diary.toast = toast;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (diary.toast = null), ms);
}

/** A sober recognition phrase from content/riconoscimenti.it.txt. */
export function recognition(): string {
  lastPhrase = pickPhrase(PHRASES, lastPhrase) ?? 'Fatto.';
  return lastPhrase;
}
