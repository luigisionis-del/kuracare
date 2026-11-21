import { describe, expect, it } from 'vitest';
import { KINDS, emptyCheckin, positive, score } from '../../src/core/events/kinds';
import { addEvent, loadEvents, removeEvent, savePhoto, updateEvent, type AnyEvent } from '../../src/core/events/repo';
import { enrichCandidates, missingDetails } from '../../src/core/events/enrich';
import { parsePhrases, pickPhrase } from '../../src/core/events/recognition';
import { defaultLists } from '../../src/core/events/lists';
import { fromLocalInput, toLocalInput, isSameLocalDay } from '../../src/core/model/localtime';
import { newEvent } from '../../src/core/model/event';
import { freshVault } from './helpers';

describe('values', () => {
  it('score keeps 0-10 integers and turns skips into null', () => {
    expect(score(7)).toBe(7);
    expect(score(0)).toBe(0);
    expect(score(12)).toBe(10);
    expect(score(null)).toBeNull();
    expect(score('')).toBeNull();
    expect(score('x')).toBeNull();
  });

  it('positive accepts Italian decimal commas', () => {
    expect(positive('0,5')).toBe(0.5);
    expect(positive('22')).toBe(22);
    expect(positive('-1')).toBeNull();
    expect(positive('150', 100)).toBeNull();
    expect(positive('')).toBeNull();
  });
});

describe('research allow-list', () => {
  it('never includes free text or photos', () => {
    const forbidden = ['note', 'text', 'description', 'photoId', 'presetId'];
    for (const def of Object.values(KINDS)) for (const f of forbidden) expect(def.research).not.toContain(f);
    expect(KINDS.reflection.research).toEqual([]);
  });
});

describe('lists', () => {
  it('default presets carry no invented values', () => {
    const l = defaultLists();
    for (const s of l.strains) {
      expect(s.thcPct).toBeNull();
      expect(s.cbdPct).toBeNull();
      expect(s.amount).toBeNull();
    }
    for (const s of l.supplements) expect(s.amount).toBeNull();
  });
});

describe('repo', () => {
  it('adds, loads newest first, updates and removes', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    const a = await addEvent(v, 'checkin', { ...emptyCheckin(), mood: 6 }, { when: new Date(Date.now() - 60_000) });
    const b = await addEvent(v, 'care', { label: 'Passeggiata', durationMin: null, fulfilment: null });
    let all = await loadEvents(v);
    expect(all.map((e) => e.id)).toEqual([b.id, a.id]);
    expect(a.origin).toBe('spontaneous');
    expect(a.payload.energy).toBeNull();

    const u = await updateEvent(v, { ...b, payload: { ...b.payload, fulfilment: 8 } });
    expect(u.editedAt).toBeTruthy();
    expect(u.recordedAt).toBe(b.recordedAt);
    all = await loadEvents(v);
    expect(all.find((e) => e.id === b.id)?.payload).toMatchObject({ fulfilment: 8 });

    await removeEvent(v, a);
    expect((await loadEvents(v)).length).toBe(1);
  });

  it('removing a meal removes its photo too', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    const photoId = await savePhoto(v, 'data:image/jpeg;base64,AAAA');
    const m = await addEvent(v, 'meal', { mealType: 'lunch', description: null, photoId });
    await removeEvent(v, m);
    expect(await v.get(photoId)).toBeUndefined();
  });

  it('upgrades old payloads when reading', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    const e = newEvent('care', 1, { label: 'Natura', durationMin: 20, fulfilment: null });
    await v.put(e.id, e);
    expect((await loadEvents(v))[0]?.schemaVersion).toBe(KINDS.care.version);
  });
});

describe('enrich ("Se ti va")', () => {
  const ev = <T>(kind: string, payload: T, daysAgo = 0) =>
    newEvent(kind, 1, payload, 'spontaneous', new Date(Date.now() - daysAgo * 86_400_000)) as AnyEvent;

  it('offers factual details, never check-in answers', () => {
    expect(missingDetails(ev('checkin', emptyCheckin()))).toEqual([]);
    expect(missingDetails(ev('cannabis', { presetId: null, strain: 'X', thcPct: null, cbdPct: 1, amount: 0.1, unit: 'g', route: 'oral' }))).toEqual(['strain-values']);
    expect(missingDetails(ev('meal', { mealType: 'lunch', description: null, photoId: null }))).toEqual(['meal-what']);
  });

  it('only recent, not settled, at most 5', () => {
    const care = (d: number, settled = false) => ({ ...ev('care', { label: 'X', durationMin: null, fulfilment: null }, d), settled });
    const list = [care(0), care(1, true), care(2), care(3), care(4), care(5), care(6), care(10)];
    const out = enrichCandidates(list);
    expect(out.length).toBe(5);
    expect(out.some((e) => e.settled)).toBe(false);
  });
});

describe('recognition phrases', () => {
  it('ignores comments and blank lines, avoids repeating', () => {
    const p = parsePhrases('# commento\nUno\n\n  Due  \n');
    expect(p).toEqual(['Uno', 'Due']);
    for (let i = 0; i < 20; i++) expect(pickPhrase(p, 'Uno')).toBe('Due');
    expect(pickPhrase([], null)).toBeNull();
  });
});

describe('local time inputs', () => {
  it('round-trips through datetime-local', () => {
    const iso = fromLocalInput('2026-11-21T09:30')!;
    expect(toLocalInput(iso)).toBe('2026-11-21T09:30');
    expect(isSameLocalDay(iso, new Date(2026, 10, 21))).toBe(true);
    expect(fromLocalInput('nonsense')).toBeNull();
  });
});
