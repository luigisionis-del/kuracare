import { describe, expect, it } from 'vitest';
import { isoWithOffset } from '../../src/core/model/time';
import { newEvent } from '../../src/core/model/event';
import { MigrationRegistry } from '../../src/core/migrations';
import { isValidWindow } from '../../src/core/settings';

describe('time', () => {
  it('formats ISO 8601 with an explicit offset', () => {
    const s = isoWithOffset(new Date());
    expect(s).toMatch(/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}[+-]\d\d:\d\d$/);
    expect(new Date(s).getTime()).toBeLessThanOrEqual(Date.now());
  });

  it('round-trips the same instant', () => {
    const d = new Date('2026-03-29T00:30:00Z'); // around the EU DST switch
    expect(new Date(isoWithOffset(d)).getTime()).toBe(d.getTime());
  });
});

describe('event', () => {
  it('builds an envelope with origin, versions and zone', () => {
    const e = newEvent('checkin', 1, { umore: null }, 'prompted');
    expect(e.id).toMatch(/^evt:/);
    expect(e.origin).toBe('prompted');
    expect(e.schemaVersion).toBe(1);
    expect(e.tz).toBeTruthy();
    expect(e.payload.umore).toBeNull(); // missing stays missing
  });
});

describe('migrations', () => {
  const reg = new MigrationRegistry().register('demo', 3, {
    1: (p) => ({ ...(p as object), b: 0 }),
    2: (p) => {
      const { a, ...rest } = p as { a: number };
      return { ...rest, renamed: a };
    },
  });

  it('upgrades step by step without touching the instrument version', () => {
    const e = newEvent('demo', 1, { a: 5, itemSet: 'core-v0.1' });
    const up = reg.upgrade(e);
    expect(up.schemaVersion).toBe(3);
    expect(up.payload).toEqual({ renamed: 5, b: 0, itemSet: 'core-v0.1' });
    expect(e.payload).toEqual({ a: 5, itemSet: 'core-v0.1' }); // pure
  });

  it('refuses gaps and future versions', () => {
    expect(() => new MigrationRegistry().register('x', 2)).toThrow(/Missing migration/);
    expect(() => reg.upgrade(newEvent('demo', 9, {}))).toThrow(/newer/);
  });
});

describe('settings', () => {
  it('validates time windows', () => {
    expect(isValidWindow({ start: '09:00', end: '11:00' })).toBe(true);
    expect(isValidWindow({ start: '11:00', end: '09:00' })).toBe(false);
    expect(isValidWindow({ start: '9:00', end: '11:00' })).toBe(false);
  });
});
